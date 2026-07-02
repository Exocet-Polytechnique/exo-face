#[macro_use]
extern crate rocket;

use std::sync::{Arc, Mutex};
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

mod boat_data;
use boat_data::{BoatData, ModuleData};

mod dbc {
    #![allow(unused, non_snake_case, non_camel_case_types, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/dbc_gen.rs"));
}

// TODO: set these to the actual data_type values defined in your protocol
mod dtype {
    pub const SPEED: u8          = 0x01;
    pub const HYDROGEN_LEVEL: u8 = 0x02;
    pub const VOLTAGE: u8        = 0x10;
    pub const CURRENT: u8        = 0x11;
    pub const TEMPERATURE: u8    = 0x12;
}

// How often the WebSocket sender pushes a snapshot to the frontend
const SEND_INTERVAL_MS: u64 = 1000;

type SharedState = Arc<Mutex<BoatData>>;
type LogSender = StdArc<Sender<(String, Value)>>;

// Extract (dest_module, data_type, raw_data) only from d-frames; returns None for all other variants.
fn extract_d_frame(msg: dbc::Messages) -> Option<(u8, u8, f32)> {
    match msg {
        dbc::Messages::FrameP0d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw() as f32)),
        dbc::Messages::FrameP1d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw() as f32)),
        dbc::Messages::FrameP2d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw() as f32)),
        dbc::Messages::FrameP3d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw() as f32)),
        dbc::Messages::FrameP4d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw() as f32)),
        dbc::Messages::FrameP5d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw() as f32)),
        dbc::Messages::FrameP6d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw() as f32)),
        dbc::Messages::FrameP7d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw() as f32)),
        dbc::Messages::FrameP8d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw() as f32)),
        _ => None,
    }
}

// Apply a decoded d-frame to the shared BoatData.
// dest_module is 1-indexed; module 0 carries global boat data.
// TODO: adjust the f64/f32 bit reinterpretation if your protocol uses a different encoding.
fn apply_d_frame(state: &mut BoatData, dest_module: u8, data_type: u8, raw: f32) {
    match (dest_module, data_type) {
        (0, dtype::SPEED) => state.speed = raw as u32,
        (0, dtype::HYDROGEN_LEVEL) => state.hydrogen_level = raw as u32,
        (module, dtype::VOLTAGE) | (module, dtype::CURRENT) | (module, dtype::TEMPERATURE) => {
            let idx = module as usize;
            if idx == 0 { return; }
            // Grow the Vec so slot `idx` exists; each slot's id equals its Module enum value
            while state.modules.len() <= idx {
                let id: u32 = state.modules.len() as u32;
                state.modules.push(ModuleData { id, ..ModuleData::default() });
            }
            let m = &mut state.modules[idx];
            match data_type {
                dtype::VOLTAGE     => m.voltage = raw,
                dtype::CURRENT     => m.current = raw,
                dtype::TEMPERATURE => m.temperature = raw,
                _ => {}
            }
        }
        _ => {}
    }
}

// --- Task 1 (CAN receiver thread): reads d-frames and updates shared state ---
fn spawn_can_receiver(state: SharedState, log_tx: Sender<(String, Value)>) {
    std::thread::spawn(move || {
        let socket = CanSocket::open("can0").expect("Failed to open can0");
        let start = std::time::Instant::now();
        let msg = "CAN receiver: listening on can0 for d-frames...";
        eprintln!("{}", msg);
        let _ = log_tx.blocking_send(("info".to_string(), serde_json::json!({"msg": msg})));

        loop {
            match socket.read_frame() {
                Ok(CanFrame::Data(frame)) => {
                    let id = frame.raw_id();
                    if let Ok(msg) = dbc::Messages::from_can_message(id, frame.data()) {
                        if let Some((dest_module, data_type, raw)) = extract_d_frame(msg) {
                            let uptime_ms = start.elapsed().as_millis();
                            let _ = log_tx.blocking_send(("info".to_string(), serde_json::json!({
                                "uptime_ms": uptime_ms,
                                "msg": format!("CAN frame: id=0x{:X} dest_module={} data_type=0x{:02X} raw=0x{:X}", id, dest_module, data_type, raw)
                            })));
                            let mut s = state.lock().unwrap();
                            apply_d_frame(&mut s, dest_module, data_type, raw);
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


    // // inject test data
    // {
    //     let mut s = state.lock().unwrap();
    //     apply_d_frame(&mut s, 0, dtype::SPEED, f64::to_bits(42.5));
    //     apply_d_frame(&mut s, 1, dtype::VOLTAGE, 3700);
    //     apply_d_frame(&mut s, 1, dtype::TEMPERATURE, f32::to_bits(25.3) as u64);
    // }

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

    // start CAN receiver and pass the log sender
    spawn_can_receiver(Arc::clone(&state), (*log_tx_arc).clone());

    rocket::build()
        .mount("/", routes![stream])
        .manage(state)
        .manage(log_tx_arc)
}


enum Module {
    Broadcast = 0b1111,
    Cockpit = 0b0000,
    Hydrogen = 0b0001,
    HighPower = 0b0010,
    Sensors = 0b0011,
    CoolingSystem = 0b0100,
    IsolationControl = 0b0101,
    Dashboard = 0b0110,
    Telemetry = 0b0111,
} 
