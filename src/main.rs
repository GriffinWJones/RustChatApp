use tokio::io::{AsyncReadExt, AsyncWriteExt}; // Async read and write for network sockets
use tokio::net::TcpListener; // Allows sever to listen for TCP connections

#[tokio::main] // Initalizes tokio runtine, allows main to be async
async fn main() -> tokio::io::Result<()> {
    // Makes main async, returns a tokio::io::Result<()>
    let listener = TcpListener::bind("127.0.0.1:8080").await?; // Binders a tcp listener to ip and port and waits until binding is complete
    println!("Server listening on port 8080...");

    loop {
        let (mut socket, addr) = listener.accept().await?; //waits for a client connect and gets the socket and the client's address
        println!("New client connected: {}", addr);

        tokio::spawn(async move {
            // tokio::spawn Creates an async task (light weight thread) for client connection, move ensures ownwership of socket and addr in the task
            // This means that main cannot use socket and addr as they are now owned by the task and not main
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
