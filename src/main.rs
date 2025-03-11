use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on port 8080...");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New client connected: {}", addr);

        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => {
                        println!("Client {} disconnected", addr);
                        break;
                    }
                    Ok(n) => {
                        if let Err(e) = socket.write_all(&buffer[..n]).await {
                            eprintln!("Failed to write to client {}: {}", addr, e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from client {}: {}", addr, e);
                        break;
                    }
                }
            }
        });
    }
}
