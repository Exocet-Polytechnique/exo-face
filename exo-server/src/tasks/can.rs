use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket, StandardId};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::dbc;
use crate::boat_data::Alert;
use crate::SharedState;

// Module addresses, per exo-can/exo_can.dbc's `TargetModule` numbering (shared by every PCB).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Broadcast = 0,
    Cockpit = 1,
    Hydrogen = 2,
    TelemetryBattery = 3,
    HighPower = 4,
    DriverInterface = 5,
}

impl Module {
    fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            0 => Some(Module::Broadcast),
            1 => Some(Module::Cockpit),
            2 => Some(Module::Hydrogen),
            3 => Some(Module::TelemetryBattery),
            4 => Some(Module::HighPower),
            5 => Some(Module::DriverInterface),
            _ => None,
        }
    }
}

// Shared raw values of the `Command` signal (LP_PCBnn_P, MessageType M2), same on every PCB.
pub mod command {
    pub const START: u8 = 0;
    pub const SHUTDOWN: u8 = 1;
    #[allow(dead_code)] // not sent by us yet; reacted to as an incoming command from other PCBs
    pub const FORCE_SHUTDOWN: u8 = 2;
}

// Shared raw values of the `CurrentState` signal (LP_PCBnn_P, MessageType M1).
// DriverInterfaceHAT's own state doubles as the overall boat state.
pub mod state {
    pub const IDLE: u8 = 0;
    #[allow(dead_code)] // not broadcast yet; STARTED/IDLE cover the current handshake
    pub const STARTING: u8 = 1;
    pub const STARTED: u8 = 2;
    #[allow(dead_code)]
    pub const SHUTTING_DOWN: u8 = 3;
}

// Raw values of the `ErrorType` signal (HP_PCBnn_E), per exo_can.dbc's VAL_ table.
pub mod error {
    #[allow(dead_code)]
    pub const SYSTEM_FAULT: u16 = 0;
    pub const CAN_FAULT: u16 = 1;
}

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

// Every incoming CAN frame, from any PCB, collapses into one of these.
#[derive(Debug)]
pub enum CanRxEvent {
    Error { source: Module, critical: bool, code: u16 },
    ProcedureStatus { source: Module, status_raw: u8, kind_raw: u8 },
    ProcedureState { source: Module, state_raw: u8 },
    ProcedureCommand { source: Module, target: Module, command_raw: u8 },
    Data { source: Module },
}

// Normalizes any of the 18 generated message types into a `CanRxEvent`.
pub fn categorize(msg: dbc::Messages) -> Option<CanRxEvent> {
    use dbc::Messages::*;

    macro_rules! error_event {
        ($m:expr, $source:expr, $critical:expr, $code_fn:ident) => {
            Some(CanRxEvent::Error { source: $source, critical: $critical, code: $m.$code_fn() as u16 })
        };
    }

    // Each PCB's M0/M1/M2 structs are independent generated types, but all expose the same
    // `_raw()` accessors, so this helper is written once per PCB rather than once per signal.
    match msg {
        HpPcb01E(m) => error_event!(m, Module::Cockpit, true, error_type_raw),
        LpPcb01E(m) => error_event!(m, Module::Cockpit, false, warning_type_raw),
        LpPcb01P(mut m) => match m.message_type() {
            Ok(dbc::LpPcb01PMessageType::M0(s)) => Some(CanRxEvent::ProcedureStatus {
                source: Module::Cockpit, status_raw: s.procedure_status_raw(), kind_raw: s.procedure_type_raw(),
            }),
            Ok(dbc::LpPcb01PMessageType::M1(s)) => Some(CanRxEvent::ProcedureState {
                source: Module::Cockpit, state_raw: s.current_state_raw(),
            }),
            Ok(dbc::LpPcb01PMessageType::M2(s)) => Some(CanRxEvent::ProcedureCommand {
                source: Module::Cockpit,
                target: Module::from_raw(s.target_module_raw())?,
                command_raw: s.command_raw(),
            }),
            Err(_) => None,
        },

        HpPcb02E(m) => error_event!(m, Module::Hydrogen, true, error_type_raw),
        LpPcb02E(m) => error_event!(m, Module::Hydrogen, false, warning_type_raw),
        LpPcb02P(mut m) => match m.message_type() {
            Ok(dbc::LpPcb02PMessageType::M0(s)) => Some(CanRxEvent::ProcedureStatus {
                source: Module::Hydrogen, status_raw: s.procedure_status_raw(), kind_raw: s.procedure_type_raw(),
            }),
            Ok(dbc::LpPcb02PMessageType::M1(s)) => Some(CanRxEvent::ProcedureState {
                source: Module::Hydrogen, state_raw: s.current_state_raw(),
            }),
            Ok(dbc::LpPcb02PMessageType::M2(s)) => Some(CanRxEvent::ProcedureCommand {
                source: Module::Hydrogen,
                target: Module::from_raw(s.target_module_raw())?,
                command_raw: s.command_raw(),
            }),
            Err(_) => None,
        },
        LpPcb02D(_) => Some(CanRxEvent::Data { source: Module::Hydrogen }),

        HpPcb03E(m) => error_event!(m, Module::TelemetryBattery, true, error_type_raw),
        LpPcb03E(m) => error_event!(m, Module::TelemetryBattery, false, warning_type_raw),
        LpPcb03P(mut m) => match m.message_type() {
            Ok(dbc::LpPcb03PMessageType::M0(s)) => Some(CanRxEvent::ProcedureStatus {
                source: Module::TelemetryBattery, status_raw: s.procedure_status_raw(), kind_raw: s.procedure_type_raw(),
            }),
            Ok(dbc::LpPcb03PMessageType::M1(s)) => Some(CanRxEvent::ProcedureState {
                source: Module::TelemetryBattery, state_raw: s.current_state_raw(),
            }),
            Ok(dbc::LpPcb03PMessageType::M2(s)) => Some(CanRxEvent::ProcedureCommand {
                source: Module::TelemetryBattery,
                target: Module::from_raw(s.target_module_raw())?,
                command_raw: s.command_raw(),
            }),
            Err(_) => None,
        },
        LpPcb03D(_) => Some(CanRxEvent::Data { source: Module::TelemetryBattery }),

        HpPcb04E(m) => error_event!(m, Module::HighPower, true, error_type_raw),
        LpPcb04E(m) => error_event!(m, Module::HighPower, false, warning_type_raw),
        LpPcb04P(mut m) => match m.message_type() {
            Ok(dbc::LpPcb04PMessageType::M0(s)) => Some(CanRxEvent::ProcedureStatus {
                source: Module::HighPower, status_raw: s.procedure_status_raw(), kind_raw: s.procedure_type_raw(),
            }),
            Ok(dbc::LpPcb04PMessageType::M1(s)) => Some(CanRxEvent::ProcedureState {
                source: Module::HighPower, state_raw: s.current_state_raw(),
            }),
            Ok(dbc::LpPcb04PMessageType::M2(s)) => Some(CanRxEvent::ProcedureCommand {
                source: Module::HighPower,
                target: Module::from_raw(s.target_module_raw())?,
                command_raw: s.command_raw(),
            }),
            Err(_) => None,
        },
        LpPcb04D(_) => Some(CanRxEvent::Data { source: Module::HighPower }),

        HpPcb05E(m) => error_event!(m, Module::DriverInterface, true, error_type_raw),
        LpPcb05E(m) => error_event!(m, Module::DriverInterface, false, warning_type_raw),
        LpPcb05P(mut m) => match m.message_type() {
            Ok(dbc::LpPcb05PMessageType::M0(s)) => Some(CanRxEvent::ProcedureStatus {
                source: Module::DriverInterface, status_raw: s.procedure_status_raw(), kind_raw: s.procedure_type_raw(),
            }),
            Ok(dbc::LpPcb05PMessageType::M1(s)) => Some(CanRxEvent::ProcedureState {
                source: Module::DriverInterface, state_raw: s.current_state_raw(),
            }),
            Ok(dbc::LpPcb05PMessageType::M2(s)) => Some(CanRxEvent::ProcedureCommand {
                source: Module::DriverInterface,
                target: Module::from_raw(s.target_module_raw())?,
                command_raw: s.command_raw(),
            }),
            Err(_) => None,
        },
    }
}

// --- Sending: DriverInterfaceHAT (this binary) is only ever the legitimate transmitter of
// LP_PCB05_P (procedure) and HP_PCB05_E (its own error reports). There is intentionally no
// send_data — we never originate sensor telemetry.

fn send_lp_pcb05_p(socket: &CanSocket, build: impl FnOnce() -> Result<dbc::LpPcb05P, dbc::CanError>) -> std::io::Result<()> {
    let frame = build().map_err(|_| std::io::Error::other("invalid LP_PCB05_P parameters"))?;
    let id = StandardId::new(dbc::LpPcb05P::MESSAGE_ID as u16)
        .ok_or_else(|| std::io::Error::other("invalid LP_PCB05_P id"))?;
    let can_frame = CanFrame::new(id, frame.raw())
        .ok_or_else(|| std::io::Error::other("failed to build CAN frame"))?;
    socket.write_frame(&can_frame)
}

pub fn send_state(socket: &CanSocket, state_raw: u8) -> std::io::Result<()> {
    send_lp_pcb05_p(socket, || {
        let mut m1 = dbc::LpPcb05PMessageTypeM1::new();
        m1.set_current_state(state_raw)?;
        let mut frame = dbc::LpPcb05P::new(1)?;
        frame.set_m1(m1)?;
        Ok(frame)
    })
}

#[allow(dead_code)] // rounds out procedure-frame sending; no PCB other than Cockpit is orchestrated yet
pub fn send_command(socket: &CanSocket, target: Module, command_raw: u8) -> std::io::Result<()> {
    if target == Module::Hydrogen {
        return Err(std::io::Error::other("Hydrogen PCB is no longer used"));
    }
    send_lp_pcb05_p(socket, || {
        let mut m2 = dbc::LpPcb05PMessageTypeM2::new();
        m2.set_target_module(target as u8)?;
        m2.set_command(command_raw)?;
        let mut frame = dbc::LpPcb05P::new(2)?;
        frame.set_m2(m2)?;
        Ok(frame)
    })
}

#[allow(dead_code)] // rounds out procedure-frame sending; not needed by the Cockpit handshake yet
pub fn send_procedure_status(socket: &CanSocket, status_raw: u8, kind_raw: u8) -> std::io::Result<()> {
    send_lp_pcb05_p(socket, || {
        let mut m0 = dbc::LpPcb05PMessageTypeM0::new();
        m0.set_procedure_status(status_raw)?;
        m0.set_procedure_type(kind_raw)?;
        let mut frame = dbc::LpPcb05P::new(0)?;
        frame.set_m0(m0)?;
        Ok(frame)
    })
}

// Announces our own detected fault (e.g. a PCB not confirming a state change) via HP_PCB05_E.
// There's no destination field on error frames — this is self-originated, not addressed at
// whichever PCB caused it.
pub fn send_error(socket: &CanSocket, error_type_raw: u16) -> std::io::Result<()> {
    let frame = dbc::HpPcb05E::new(error_type_raw)
        .map_err(|_| std::io::Error::other("invalid HP_PCB05_E parameters"))?;
    let id = StandardId::new(dbc::HpPcb05E::MESSAGE_ID as u16)
        .ok_or_else(|| std::io::Error::other("invalid HP_PCB05_E id"))?;
    let can_frame = CanFrame::new(id, frame.raw())
        .ok_or_else(|| std::io::Error::other("failed to build CAN frame"))?;
    socket.write_frame(&can_frame)
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

fn source_of(event: &CanRxEvent) -> Module {
    match *event {
        CanRxEvent::Error { source, .. }
        | CanRxEvent::ProcedureStatus { source, .. }
        | CanRxEvent::ProcedureState { source, .. }
        | CanRxEvent::ProcedureCommand { source, .. }
        | CanRxEvent::Data { source } => source,
    }
}

// Display name for the PCB alert panel. Hydrogen/Broadcast never reach here in practice.
fn display_name(m: Module) -> &'static str {
    match m {
        Module::Cockpit => "Cockpit",
        Module::TelemetryBattery => "Batterie de Télémétrie",
        Module::HighPower => "Haute Puissance",
        Module::DriverInterface => "Interface Pilote",
        Module::Hydrogen | Module::Broadcast => "Inconnu",
    }
}

// Human-readable titles for every (source, critical, code) combination defined in
// exo_can.dbc's ErrorType/WarningType VAL_ tables (Hydrogen omitted — no longer used).
fn error_title(source: Module, critical: bool, code: u16) -> &'static str {
    match (source, critical, code) {
        (Module::Cockpit, true, 0) => "Défaut du bus CAN",
        (Module::Cockpit, true, 1) => "Défaut matériel",
        (Module::Cockpit, false, 0) => "Délai CAN dépassé",

        (Module::TelemetryBattery, true, 0) => "Défaut de la batterie",
        (Module::TelemetryBattery, true, 1) => "Surchauffe",
        (Module::TelemetryBattery, true, 2) => "Défaut du commutateur d'alimentation",
        (Module::TelemetryBattery, false, 0) => "Charge faible",
        (Module::TelemetryBattery, false, 1) => "Avertissement de température",

        (Module::HighPower, true, 0) => "Température batterie auxiliaire",
        (Module::HighPower, true, 1) => "Surutilisation pile à combustible",
        (Module::HighPower, true, 2) => "Exception pile à combustible",
        (Module::HighPower, true, 3) => "Surutilisation MPPT",
        (Module::HighPower, true, 4) => "Exception MPPT",
        (Module::HighPower, true, 5) => "Défaut d'isolation (IMD)",
        (Module::HighPower, false, 0) => "Avertissement température batterie auxiliaire",
        (Module::HighPower, false, 1) => "Avertissement puissance pile à combustible",
        (Module::HighPower, false, 2) => "Avertissement MPPT",
        (Module::HighPower, false, 3) => "Avertissement isolation (IMD)",

        (Module::DriverInterface, true, 0) => "Défaut système",
        (Module::DriverInterface, true, 1) => "Défaut CAN",
        (Module::DriverInterface, false, 0) => "Avertissement de communication",

        _ => "Erreur inconnue",
    }
}

// Updates `source`'s row in the PCB alert panel with its latest reported error/warning.
fn update_alert(state: &SharedState, source: Module, critical: bool, code: u16) {
    let mut data = state.lock().unwrap();
    if let Some(pcb) = data.pcb_status.iter_mut().find(|p| p.name == display_name(source)) {
        pcb.alert = Some(Alert {
            title: error_title(source, critical, code).to_string(),
            severity: if critical { "error" } else { "warning" }.to_string(),
        });
    }
}

// Decodes the battery signals the UI cares about straight off the raw message, since
// `categorize` only tags data frames with their source, not their decoded content.
// Aux battery charge has no CAN signal yet — exo_can.dbc only defines AuxBatteryTemperature
// for that PCB, a different quantity — so `aux_battery_charge` stays at its default until a
// real signal is added.
fn update_battery_gauges(state: &SharedState, msg: &mut dbc::Messages) {
    if let dbc::Messages::LpPcb03D(m) = msg {
        if let Ok(dbc::LpPcb03DSensor::M0(s)) = m.sensor() {
            state.lock().unwrap().telemetry_battery_charge = s.batt_so_c() as f32 / 255.0 * 100.0;
        }
    }
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

                            // A PCB confirming the state change we're waiting on.
                            if let CanRxEvent::ProcedureState { source, state_raw } = event {
                                if let Some(p) = &mut pending {
                                    if state_raw == p.expected_state {
                                        p.waiting_on.retain(|m| *m != source);
                                        if p.waiting_on.is_empty() {
                                            pending = None;
                                        }
                                    }
                                }
                            }

                            if let CanRxEvent::ProcedureCommand { target, command_raw, .. } = event {
                                let addressed_to_us = matches!(target, Module::DriverInterface | Module::Broadcast);
                                // Any PCB (except Hydrogen, already skipped above) commanding a
                                // start/shutdown gets the same state-change reply Cockpit gets.
                                if addressed_to_us {
                                    let reply_state = match command_raw {
                                        command::START => Some(state::STARTED),
                                        command::SHUTDOWN => Some(state::IDLE),
                                        _ => None,
                                    };
                                    if let Some(reply_state) = reply_state {
                                        if let Err(e) = send_state(&socket, reply_state) {
                                            let _ = log_tx.blocking_send(("error".to_string(), serde_json::json!({
                                                "msg": format!("Orchestrator: failed to send state reply: {}", e)
                                            })));
                                        } else {
                                            pending = Some(PendingConfirmation {
                                                expected_state: reply_state,
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
