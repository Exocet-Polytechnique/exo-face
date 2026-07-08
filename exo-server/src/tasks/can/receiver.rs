use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::dbc;
use crate::SharedState;
use super::protocol::{command, error, state, Module};
use super::categorize::{categorize, source_of, CanRxEvent};
use super::send::{send_command, send_error, send_state};
use super::dashboard_state::{update_alert, update_battery_gauges};

// PCBs we expect to confirm a state change (via their own CurrentState) after we announce one.
// Hydrogen is excluded — it's no longer used.
const EXPECTED_CONFIRMERS: [Module; 3] = [Module::Cockpit, Module::TelemetryBattery, Module::HighPower];

// How long to wait for a confirmation before treating a PCB as unresponsive.
const CONFIRMATION_TIMEOUT_MS: u64 = 3000;

// How often the receive loop wakes up even without traffic, to check confirmation deadlines.
const POLL_TICK_MS: u64 = 250;

struct PendingConfirmation {
    expected_state: u8,
    deadline: std::time::Instant,
    waiting_on: Vec<Module>,
}

fn log_rx_event(log_tx: &Sender<(String, Value)>, uptime_ms: u128, event: &CanRxEvent) {
    use CanRxEvent::*;
    let msg = match event {
        Error { source, critical, code } => format!(
            "CAN {} from {:?}: code=0x{:X}",
            if *critical { "ERROR" } else { "warning" }, source, code
        ),
        ProcedureStatus { source, status_raw, kind_raw } => format!(
            "CAN procedure status from {:?}: status=0x{:X} kind=0x{:X}", source, status_raw, kind_raw
        ),
        ProcedureState { source, state_raw } => format!(
            "CAN state from {:?}: state=0x{:X}", source, state_raw
        ),
        ProcedureCommand { source, target, command_raw } => format!(
            "CAN command from {:?} to {:?}: command=0x{:X}", source, target, command_raw
        ),
        Data { source } => format!("CAN data frame from {:?}", source),
    };
    let level = if matches!(event, Error { critical: true, .. }) { "error" } else { "info" };
    let _ = log_tx.blocking_send((level.to_string(), serde_json::json!({"uptime_ms": uptime_ms, "msg": msg})));
}

// Fires when `pending`'s deadline has passed: reports every PCB that never confirmed, via a
// self-originated HP_PCB05_E CAN frame and an application-level error log (for the future UI).
fn report_missing_confirmations(socket: &CanSocket, log_tx: &Sender<(String, Value)>, pending: &PendingConfirmation) {
    for missing in &pending.waiting_on {
        if let Err(e) = send_error(socket, error::CAN_FAULT) {
            let _ = log_tx.blocking_send(("error".to_string(), serde_json::json!({
                "msg": format!("Failed to send HP_PCB05_E for missing confirmation: {}", e)
            })));
        }
        let _ = log_tx.blocking_send(("error".to_string(), serde_json::json!({
            "msg": format!("{:?} did not confirm state change to 0x{:X} within {}ms", missing, pending.expected_state, CONFIRMATION_TIMEOUT_MS)
        })));
    }
}

// --- Task (CAN receiver thread): reads all frames, reacts to any PCB's start/shutdown command ---
// Hydrogen is no longer used: its frames are still defined in the dbc, but ignored here entirely.
pub fn spawn_can_receiver(log_tx: Sender<(String, Value)>, state: SharedState) {
    std::thread::spawn(move || {
        let socket = CanSocket::open("can0").expect("Failed to open can0");
        let start = std::time::Instant::now();
        let msg = "CAN receiver: listening on can0...";
        eprintln!("{}", msg);
        let _ = log_tx.blocking_send(("info".to_string(), serde_json::json!({"msg": msg})));

        let mut pending: Option<PendingConfirmation> = None;
        let mut boat_state: u8 = state::IDLE;

        loop {
            match socket.read_frame_timeout(std::time::Duration::from_millis(POLL_TICK_MS)) {
                Ok(CanFrame::Data(frame)) => {
                    let id = frame.raw_id();
                    if let Ok(mut msg) = dbc::Messages::from_can_message(id, frame.data()) {
                        update_battery_gauges(&state, &mut msg);
                        if let Some(event) = categorize(msg) {
                            if source_of(&event) == Module::Hydrogen {
                                continue;
                            }
                            log_rx_event(&log_tx, start.elapsed().as_millis(), &event);

                            if let CanRxEvent::Error { source, critical, code } = event {
                                update_alert(&state, source, critical, code);
                            }

                            // A PCB confirming it reached the state we're waiting on.
                            if let CanRxEvent::ProcedureState { source, state_raw } = event {
                                if let Some(p) = &mut pending {
                                    if state_raw == p.expected_state {
                                        p.waiting_on.retain(|m| *m != source);
                                        if p.waiting_on.is_empty() {
                                            // Every PCB has confirmed — only now does the boat
                                            // actually reach the target state.
                                            boat_state = p.expected_state;
                                            if let Err(e) = send_state(&socket, boat_state) {
                                                let _ = log_tx.blocking_send(("error".to_string(), serde_json::json!({
                                                    "msg": format!("Orchestrator: failed to announce state 0x{:X}: {}", boat_state, e)
                                                })));
                                            }
                                            pending = None;
                                        }
                                    }
                                }
                            }

                            if let CanRxEvent::ProcedureCommand { source, target, command_raw } = event {
                                let addressed_to_us = matches!(target, Module::DriverInterface | Module::Broadcast);
                                // Any PCB (except Hydrogen, already skipped above) commanding a
                                // start/shutdown gets the same reaction Cockpit gets. Only valid
                                // from the state it actually applies to, so a stray/duplicate
                                // command can't restart an already in-flight transition.
                                if addressed_to_us {
                                    let transition = match command_raw {
                                        command::START if boat_state == state::IDLE => {
                                            Some((state::STARTING, state::STARTED))
                                        }
                                        command::SHUTDOWN if boat_state == state::STARTED => {
                                            Some((state::SHUTTING_DOWN, state::IDLE))
                                        }
                                        _ => None,
                                    };
                                    if let Some((intermediate_state, final_state)) = transition {
                                        if let Err(e) = send_state(&socket, intermediate_state) {
                                            let _ = log_tx.blocking_send(("error".to_string(), serde_json::json!({
                                                "msg": format!("Orchestrator: failed to announce state 0x{:X}: {}", intermediate_state, e)
                                            })));
                                        } else {
                                            boat_state = intermediate_state;
                                            // Relay the same command to every other PCB we
                                            // orchestrate, so they actually act on it — the
                                            // one that originated the request already knows.
                                            for &pcb in EXPECTED_CONFIRMERS.iter().filter(|&&m| m != source) {
                                                if let Err(e) = send_command(&socket, pcb, command_raw) {
                                                    let _ = log_tx.blocking_send(("error".to_string(), serde_json::json!({
                                                        "msg": format!("Orchestrator: failed to relay command to {:?}: {}", pcb, e)
                                                    })));
                                                }
                                            }
                                            // The boat only actually reaches `final_state` once
                                            // every PCB confirms it got there too (see the
                                            // ProcedureState handling above).
                                            pending = Some(PendingConfirmation {
                                                expected_state: final_state,
                                                deadline: std::time::Instant::now() + std::time::Duration::from_millis(CONFIRMATION_TIMEOUT_MS),
                                                waiting_on: EXPECTED_CONFIRMERS.to_vec(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    let msg = format!("CAN read error: {}", e);
                    eprintln!("{}", msg);
                    let _ = log_tx.blocking_send(("error".to_string(), serde_json::json!({"msg": msg})));
                }
            }

            if let Some(p) = &pending {
                if std::time::Instant::now() >= p.deadline {
                    report_missing_confirmations(&socket, &log_tx, p);
                    pending = None;
                }
            }
        }
    });
}
