#[macro_use]
extern crate rocket;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU8, Ordering};
use rocket::futures::SinkExt;
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket};
use serde_json::Value;
use tokio::sync::mpsc::{Sender};
use std::path::PathBuf;
use std::env;
use std::sync::Arc as StdArc;
use futures::stream::StreamExt;

mod logging;
mod ssh_uploader;
mod orchestrator;

mod boat_data;
use boat_data::BoatData;

mod dbc {
    #![allow(unused, non_snake_case, non_camel_case_types, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/dbc_gen.rs"));
}

// How often the WebSocket sender pushes a snapshot to the frontend
const SEND_INTERVAL_MS: u64 = 1000;

// How often the dashboard broadcasts its own CurrentState (LP_PCB05_P M1) as a liveness heartbeat.
const HEARTBEAT_INTERVAL_MS: u64 = 2000;

type SharedState = Arc<Mutex<BoatData>>;
type LogSender = StdArc<Sender<(String, Value)>>;

fn log_rx_event(log_tx: &Sender<(String, Value)>, uptime_ms: u128, event: &orchestrator::CanRxEvent) {
    use orchestrator::CanRxEvent::*;
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

// --- Task 1 (CAN receiver thread): reads all frames, reacts to the Cockpit start/shutdown handshake ---
fn spawn_can_receiver(log_tx: Sender<(String, Value)>, boat_state: Arc<AtomicU8>) {
    std::thread::spawn(move || {
        let socket = CanSocket::open("can0").expect("Failed to open can0");
        let start = std::time::Instant::now();
        let msg = "CAN receiver: listening on can0...";
        eprintln!("{}", msg);
        let _ = log_tx.blocking_send(("info".to_string(), serde_json::json!({"msg": msg})));

        loop {
            match socket.read_frame() {
                Ok(CanFrame::Data(frame)) => {
                    let id = frame.raw_id();
                    if let Ok(msg) = dbc::Messages::from_can_message(id, frame.data()) {
                        if let Some(event) = orchestrator::categorize(msg) {
                            log_rx_event(&log_tx, start.elapsed().as_millis(), &event);

                            if let orchestrator::CanRxEvent::ProcedureCommand { source, target, command_raw } = event {
                                let addressed_to_us = matches!(target, orchestrator::Module::DriverInterface | orchestrator::Module::Broadcast);
                                // If the command is addressed to the DriverInterface or is a broadcast
                                if source == orchestrator::Module::Cockpit && addressed_to_us {
                                    let reply_state = match command_raw {
                                        orchestrator::command::START => Some(orchestrator::state::STARTED),
                                        orchestrator::command::SHUTDOWN => Some(orchestrator::state::IDLE),
                                        _ => None,
                                    };
                                    if let Some(reply_state) = reply_state {
                                        // Store immediately so the heartbeat thread picks it up even if
                                        // this direct reply is lost; also send right away for a fast ack
                                        // instead of waiting up to HEARTBEAT_INTERVAL_MS for the next tick.
                                        boat_state.store(reply_state, Ordering::Relaxed);
                                        if let Err(e) = orchestrator::send_state(&socket, reply_state) {
                                            let _ = log_tx.blocking_send(("error".to_string(), serde_json::json!({
                                                "msg": format!("Orchestrator: failed to send state reply: {}", e)
                                            })));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    let msg = format!("CAN read error: {}", e);
                    eprintln!("{}", msg);
                    let _ = log_tx.blocking_send(("error".to_string(), serde_json::json!({"msg": msg})));
                }
            }
        }
    });
}

// --- Task 1b (heartbeat thread): periodically broadcasts our own CurrentState (LP_PCB05_P M1)
// so every PCB can tell the dashboard is alive and knows the current boat mode.
fn spawn_heartbeat(boat_state: Arc<AtomicU8>, log_tx: Sender<(String, Value)>) {
    std::thread::spawn(move || {
        let socket = CanSocket::open("can0").expect("Failed to open can0 (heartbeat)");
        loop {
            let state = boat_state.load(Ordering::Relaxed);
            if let Err(e) = orchestrator::send_state(&socket, state) {
                let _ = log_tx.blocking_send(("error".to_string(), serde_json::json!({
                    "msg": format!("Heartbeat: failed to send state: {}", e)
                })));
            }
            std::thread::sleep(std::time::Duration::from_millis(HEARTBEAT_INTERVAL_MS));
        }
    });
}

// --- Task 2 (WebSocket sender & receiver): pushes JSON snapshot and receives frontend logs ---
#[get("/")]
fn stream(ws: ws::WebSocket, state: &rocket::State<SharedState>, log_tx: &rocket::State<LogSender>) -> ws::Channel<'static> {
    let state = Arc::clone(state);
    let log_tx = Arc::clone(&**log_tx);
    ws.channel(move |mut stream| {
        Box::pin(async move {
            let mut ticker = rocket::tokio::time::interval(std::time::Duration::from_millis(SEND_INTERVAL_MS));

            loop {
                rocket::tokio::select! {
                    _ = ticker.tick() => {
                        let json = {
                            let s = state.lock().unwrap();
                            serde_json::to_string(&*s).unwrap()
                        };
                        if stream.send(ws::Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    msg = stream.next() => {
                        match msg {
                            Some(Ok(ws::Message::Text(text))) => {
                                if let Ok(log_msg) = serde_json::from_str::<Value>(&text) {
                                    if log_msg.get("type").and_then(|t| t.as_str()) == Some("log") {
                                        let level = log_msg.get("level").and_then(|l| l.as_str()).unwrap_or("info");
                                        let message = log_msg.get("message").and_then(|m| m.as_str()).unwrap_or("");
                                        let payload = serde_json::json!({"source": "frontend", "message": message});
                                        let _ = log_tx.send((level.to_string(), payload)).await;
                                    }
                                }
                            }
                            Some(Ok(ws::Message::Close(..))) | None | Some(Err(_)) => break,
                            _ => {}
                        }
                    }
                }
            }

            Ok(())
        })
    })
}

#[launch]
fn rocket() -> _ {
    let state: SharedState = Arc::new(Mutex::new(BoatData::default()));
    let boat_state = Arc::new(AtomicU8::new(orchestrator::state::IDLE));

    // --- Setup async logging and uploader workers in a separate tokio runtime ---
    let (log_tx, log_rx) = tokio::sync::mpsc::channel::<(String, Value)>(1024);
    let (upload_tx, upload_rx) = tokio::sync::mpsc::channel::<PathBuf>(128);

    // Read remote settings from env or defaults
    let remote_host = env::var("LOG_REMOTE_HOST").unwrap_or_else(|_| "192.168.1.100".to_string());
    let remote_user = env::var("LOG_REMOTE_USER").unwrap_or_else(|_| "pi".to_string());
    let remote_path = PathBuf::from(env::var("LOG_REMOTE_PATH").unwrap_or_else(|_| "/home/pi/logs".to_string()));

    let log_dir = PathBuf::from(env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string()));

    // Spawn a thread that runs a tokio runtime to host async workers
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        let handle = rt.handle().clone();
        rt.block_on(async move {
            // start logging worker
            handle.spawn(logging::logging_worker(log_rx, log_dir.clone()));
            // start ssh uploader worker
            handle.spawn(ssh_uploader::ssh_uploader_worker(upload_rx, remote_host.clone(), remote_user.clone(), remote_path.clone()));

            // sweeper: periodically scan log dir for files older than N seconds and send to uploader
            let sweeper_upload = upload_tx.clone();
            let sweeper_dir = log_dir.clone();
            handle.spawn(async move {
                use tokio::time::{sleep, Duration};
                loop {
                    sleep(Duration::from_secs(30)).await;
                    let today = format!("logs-{}.jsonl", chrono::Utc::now().format("%Y-%m-%d"));
                    if let Ok(mut dir) = tokio::fs::read_dir(&sweeper_dir).await {
                        while let Ok(Some(entry)) = dir.next_entry().await {
                            // never upload the current day's file — it's still being written
                            if entry.file_name().to_str() == Some(&today) { continue; }
                            if let Ok(meta) = entry.metadata().await {
                                if let Ok(mtime) = meta.modified() {
                                    if let Ok(elapsed) = mtime.elapsed() {
                                        if elapsed.as_secs() > 30 {
                                            let _ = sweeper_upload.send(entry.path()).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });

            // keep runtime alive
            futures::future::pending::<()>().await;
        });
    });

    // Wrap log_tx in Arc for Rocket management
    let log_tx_arc: LogSender = StdArc::new(log_tx);

    // start CAN receiver and the heartbeat broadcaster, sharing the current boat state between them
    spawn_can_receiver((*log_tx_arc).clone(), Arc::clone(&boat_state));
    spawn_heartbeat(boat_state, (*log_tx_arc).clone());

    rocket::build()
        .mount("/", routes![stream])
        .manage(state)
        .manage(log_tx_arc)
}
