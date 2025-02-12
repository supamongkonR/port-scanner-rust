use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
/// Starts a simple HTTP server on port 80.
async fn main() {
    // Bind the TCP listener to 0.0.0.0:80 (all interfaces, port 80).
    let listener = TcpListener::bind("0.0.0.0:80")
        .await
        .expect("Failed to bind to port 80");

    println!("Server running on port 80");

    loop {
        // Accept an incoming connection.
        let (mut socket, addr) = listener
            .accept()
            .await
            .expect("Failed to accept connection");
        println!("Accepted connection from: {}", addr);

        // Spawn a new task to handle the connection concurrently.
        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // Read data from the socket (an HTTP request).
            let bytes_read = match socket.read(&mut buf).await {
                Ok(n) if n == 0 => return, // Connection closed.
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Failed to read from socket; err = {:?}", e);
                    return;
                }
            };

            println!(
                "Received request:\n{}",
                String::from_utf8_lossy(&buf[..bytes_read])
            );

            // Prepare a simple HTTP response.
            let response = "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello World!";

            // Write the response back to the socket.
            if let Err(e) = socket.write_all(response.as_bytes()).await {
                eprintln!("Failed to write to socket; err = {:?}", e);
            }
        });
    }
}
