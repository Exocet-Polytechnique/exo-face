use anyhow::Result;
use ssh2::Session;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{PathBuf};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

/// A simple uploader which tries to connect via SSH and upload files by opening an SFTP session.
/// This runs synchronously in a spawned blocking task because `ssh2` is blocking.
pub async fn ssh_uploader_worker(mut rx: Receiver<PathBuf>, remote_host: String, remote_user: String, remote_path: PathBuf) {
    while let Some(path) = rx.recv().await {
        // Spawn a blocking task for the upload
        let host = remote_host.clone();
        let user = remote_user.clone();
        let rpath = remote_path.clone();
        tokio::task::spawn_blocking(move || {
            // Attempt several retries with backoff
            let mut attempts = 0;
            loop {
                if attempts > 0 {
                    thread::sleep(Duration::from_secs(5 * attempts.min(6)));
                }
                attempts += 1;
                match try_upload(&host, &user, &path, &rpath) {
                    Ok(_) => {
                        // On success remove local file
                        let _ = std::fs::remove_file(&path);
                        break;
                    }
                    Err(e) => {
                        eprintln!("Upload failed (attempt {}): {}", attempts, e);
                        if attempts >= 5 {
                            eprintln!("Giving up upload for {} after {} attempts", path.display(), attempts);
                            break;
                        }
                        // continue to retry
                    }
                }
            }
        });
    }
}

fn try_upload(host: &str, user: &str, local_path: &PathBuf, remote_path: &PathBuf) -> Result<()> {
    // Connect to SSH server
    let tcp = TcpStream::connect(format!("{}:22", host))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Try publickey auth from default locations
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    let privkey = home.join(".ssh/id_rsa");
    let pubkey = home.join(".ssh/id_rsa.pub");

    if privkey.exists() {
        sess.userauth_pubkey_file(user, Some(&pubkey), &privkey, None)?;
    }

    if !sess.authenticated() {
        return Err(anyhow::anyhow!("SSH authentication failed"));
    }

    let sftp = sess.sftp()?;

    // open local file
    let mut local = File::open(local_path)?;
    let mut contents = Vec::new();
    local.read_to_end(&mut contents)?;

    let remote_file_path = remote_path.join(local_path.file_name().unwrap());
    let mut remote_file = sftp.create(&remote_file_path)?;
    remote_file.write_all(&contents)?;
    Ok(())
}
