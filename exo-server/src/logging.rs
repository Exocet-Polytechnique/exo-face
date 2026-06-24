use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use std::path::{PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Receiver;

#[derive(Serialize)]
pub struct LogEntry {
    pub ts: DateTime<Utc>,
    pub level: String,
    pub payload: Value,
}

pub async fn logging_worker(mut rx: Receiver<(String, Value)>, mut dir: PathBuf)
{
    // Ensure directory exists
    let _ = tokio::fs::create_dir_all(&dir).await;

    while let Some((level, payload)) = rx.recv().await {
        let entry = LogEntry { ts: Utc::now(), level, payload };
        match serde_json::to_string(&entry) {
            Ok(mut line) => {
                line.push('\n');
                // Rotate file per day
                let filename = format!("logs-{}.jsonl", Utc::now().format("%Y-%m-%d"));
                let mut path = dir.clone();
                path.push(filename);

                // Open with append
                match OpenOptions::new().create(true).append(true).open(&path).await {
                    Ok(mut f) => {
                        if let Err(e) = f.write_all(line.as_bytes()).await {
                            eprintln!("Failed to write log: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to open log file {}: {}", path.display(), e),
                }
            }
            Err(e) => eprintln!("Failed to serialize log entry: {}", e),
        }
    }
}
