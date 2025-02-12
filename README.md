# Async Port Scanner

## Description

The **Async Port Scanner** is a Rust-based network scanning tool that leverages Tokio for asynchronous concurrency. It scans all ports on a given IP address, performs banner grabbing on open ports, and retrieves process information using macOS’s `lsof` and `ps` commands. The scanner runs in continuous cycles (every 3 seconds by default) and only notifies you when a port is newly discovered or has closed since the last scan.

## Features

- **Asynchronous Scanning:** Uses Tokio to scan ports concurrently, reducing overall scan time.
- **Banner Grabbing:** Retrieves service banners from open ports to help identify running services.
- **Process Information:** Utilizes macOS commands (`lsof` and `ps`) to extract process name, PID, and start time for services running on open ports.
- **Change Detection:** Only sends notifications for ports that are newly open or have closed since the last cycle.
- **Batch Processing:** Scans ports in batches (e.g., 100 tasks per batch) to efficiently manage system resources.
- **Environment-Based Configuration:** Customize service names via a `.env` file (using the `dotenvy` crate).

## Requirements

- **Rust** (latest stable version recommended)
- **macOS:** (for process info using `lsof` and `ps`)
- **Tokio** (asynchronous runtime)
- **Futures** crate (for joining futures in batches)
- **dotenvy** crate (for environment variable configuration)
- **lsof & ps:** These commands should be available on your system.

## Installation

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/yourusername/async-port-scanner.git
   ```
   ```bash
   cd async-port-scanner
   cargo run
   ```

## Example 
<img width="974" alt="image" src="https://github.com/user-attachments/assets/de886283-b995-486b-9b60-d62ebc6a1975" />
