use dotenvy::dotenv;
use futures::future::join_all;
use log::info;
use std::{collections::BTreeMap, env, process::Command};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::mpsc, task};

fn get_process_info(port: u16) -> String {
    // Get the PID of the scanning process.
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

fn identify_service(port: u16) -> String {
    dotenv().ok();
    let key = format!("SERVICE_{}", port);
    if let Ok(service) = env::var(key) {
        return service;
    }
    get_process_info(port)
}

pub async fn grab_banner(ip: &str, port: u16) -> Option<String> {
    let addr = format!("{}:{}", ip, port);
    if let Ok(mut stream) = TcpStream::connect(&addr).await {
        let service = identify_service(port);
        // For HTTP/HTTPS ports, send a request to trigger a response.
        if port == 80 || port == 443 {
            let _ = stream.write_all(b"HEAD / HTTP/1.1\r\n\r\n").await;
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
/// Every 3 seconds, compare the working ports to a stored set so that you only send a notification
/// (via `tx`) for ports that are newly discovered in the current scan.
pub async fn scan_ports(tx: mpsc::Sender<String>, ip: &str) {
    let mut seen_ports: BTreeMap<u16, String> = BTreeMap::new();

    loop {
        let mut new_ports: BTreeMap<u16, String> = BTreeMap::new();
        let mut tasks = Vec::new();

        for port in 1..=65535 {
            let ip_clone = ip.to_string();
            let task = task::spawn(async move {
                if let Some(banner) = grab_banner(&ip_clone, port).await {
                    Some((port, banner))
                } else {
                    None
                }
            });
            tasks.push(task);

            if tasks.len() >= 100 {
                let results = join_all(tasks.drain(..)).await;
                for res in results {
                    if let Ok(Some((port, banner))) = res {
                        new_ports.insert(port, banner);
                    }
                }
            }
        }

        if !tasks.is_empty() {
            let results = join_all(tasks.drain(..)).await;
            for res in results {
                if let Ok(Some((port, banner))) = res {
                    new_ports.insert(port, banner);
                }
            }
        }

        for (port, banner) in &new_ports {
            if !seen_ports.contains_key(port) {
                info!("New open port detected: {}: {}", port, banner);
                let _ = tx
                    .send(format!("New open port detected: {}: {}", port, banner))
                    .await;
            }
        }

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
