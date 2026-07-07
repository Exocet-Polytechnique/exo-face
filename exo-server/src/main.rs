#[macro_use]
extern crate rocket;

use std::sync::{Arc, Mutex};
use rocket::futures::SinkExt;
use serde_json::Value;
use tokio::sync::mpsc::{Sender};
use std::path::PathBuf;
use std::env;
use std::sync::Arc as StdArc;
use futures::stream::StreamExt;

pub mod tasks;

mod boat_data;
use boat_data::BoatData;

mod dbc {
    #![allow(unused, non_snake_case, non_camel_case_types, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/dbc_gen.rs"));
}

// How often the WebSocket sender pushes a snapshot to the frontend
const SEND_INTERVAL_MS: u64 = 1000;

type SharedState = Arc<Mutex<BoatData>>;
type LogSender = StdArc<Sender<(String, Value)>>;

// --- Task: WebSocket sender & receiver — pushes JSON snapshot and receives frontend logs ---
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
            handle.spawn(tasks::logging::logging_worker(log_rx, log_dir.clone()));
            // start ssh uploader worker
            handle.spawn(tasks::ssh_uploader::ssh_uploader_worker(upload_rx, remote_host.clone(), remote_user.clone(), remote_path.clone()));

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

    // start CAN receiver
    tasks::can::spawn_can_receiver((*log_tx_arc).clone());

    rocket::build()
        .mount("/", routes![stream])
        .manage(state)
        .manage(log_tx_arc)
}
