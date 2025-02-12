use dotenvy::dotenv;
use futures::stream::{self, StreamExt};
use log::info;
use std::{collections::BTreeMap, env, process::Command};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::mpsc};

/// Get process information by filtering out our own process using `lsof` and `ps`.
fn get_process_info(port: u16) -> String {
    let our_pid = std::process::id();
    // Construct a shell command that:
    // - Lists processes using the port with lsof,
    // - Filters out any line containing our own PID,
    // - Skips the header (using tail -n +2) and takes the first matching line,
    // - Uses awk to print the process name and PID,
    // - And then uses ps to get the start time of that process.
    let command = format!(
        "lsof -i :{} | grep -v {} | tail -n +2 | head -n 1 | awk '{{print $1, $2}}' && \
         ps -o lstart= -p $(lsof -i :{} | grep -v {} | tail -n +2 | head -n 1 | awk '{{print $2}}')",
        port, our_pid, port, our_pid
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("Failed to execute lsof command");

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if result.is_empty() {
        return "Unknown Process".to_string();
    }

    let mut lines = result.lines();
    let process_info = lines.next().unwrap_or("Unknown Process");
    let start_time = lines.next().unwrap_or("Unknown Start Time");

    format!("{} (Started: {})", process_info, start_time)
}

/// Identify the service running on a given port.
/// Checks environment variables via a .env file first, then falls back to system process info.
fn identify_service(port: u16) -> String {
    dotenv().ok();
    let key = format!("SERVICE_{}", port);
    if let Ok(service) = env::var(key) {
        return service;
    }
    get_process_info(port)
}

/// Attempts to grab a banner from the given IP and port.
/// For HTTP/HTTPS ports, it sends a basic HTTP HEAD request.
pub async fn grab_banner(ip: &str, port: u16) -> Option<String> {
    let addr = format!("{}:{}", ip, port);
    if let Ok(mut stream) = TcpStream::connect(&addr).await {
        let service = identify_service(port);
        if port == 80 || port == 443 {
            let _ = stream
                .write_all(b"HEAD / HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .await;
        }
        let mut buffer = [0; 1024];

        if let Ok(size) = stream.try_read(&mut buffer) {
            let banner = String::from_utf8_lossy(&buffer[..size]).to_string();
            info!("🎯 Port {} ({}) open - Banner: {}", port, service, banner);
            return Some(format!("{} ({}) -> {}", port, service, banner));
        }
        info!("🎯 Port {} ({}) open - No banner received", port, service);
        return Some(format!("{} ({}) -> No banner", port, service));
    }
    None
}

/// Scan ports using async tasks with logging.
/// Every 3 seconds, compare the current open ports with a stored set and send a notification
/// (via `tx`) for ports that are newly discovered or have closed.
pub async fn scan_ports(tx: mpsc::Sender<String>, ip: &str) {
    let mut seen_ports: BTreeMap<u16, String> = BTreeMap::new();

    loop {
        let new_ports: BTreeMap<u16, String> = stream::iter(1..=65535)
            .map(|port| {
                let ip_clone = ip.to_string();
                async move {
                    if let Some(banner) = grab_banner(&ip_clone, port).await {
                        Some((port, banner))
                    } else {
                        None
                    }
                }
            })
            .buffer_unordered(100)
            .filter_map(|res| async move { res })
            .fold(BTreeMap::new(), |mut acc, (port, banner)| async move {
                acc.insert(port, banner);
                acc
            })
            .await;

        // Notify for newly discovered open ports.
        for (port, banner) in &new_ports {
            if !seen_ports.contains_key(port) {
                info!("New open port detected: {}: {}", port, banner);
                let _ = tx
                    .send(format!("New open port detected: {}: {}", port, banner))
                    .await;
            }
        }

        // Optionally notify for ports that have closed.
        for port in seen_ports.keys() {
            if !new_ports.contains_key(port) {
                info!("Port {} closed", port);
                let _ = tx.send(format!("Port {} closed", port)).await;
            }
        }

        seen_ports = new_ports;

        info!("⏳ Waiting 3 seconds before the next scan...");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
