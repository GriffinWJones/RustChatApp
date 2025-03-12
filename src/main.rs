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

            let mut buffer = [0; 1024]; // Create buffer for store incoming data from client

            loop {
                match socket.read(&mut buffer).await {
                    // waits to read from the socket into the buffer
                    Ok(0) => {
                        //When client disconnects
                        println!("Client {} disconnected", addr);
                        break; // Break connection
                    }
                    Ok(n) => {
                        //This means n bytes were read
                        if let Err(e) = socket.write_all(&buffer[..n]).await {
                            // attempts to write everything in the buffer back to the socket (this is the echo)
                            eprintln!("Failed to write to client {}: {}", addr, e); // If there was an error writng back this runs and connection breaks
                            break;
                        }
                    }
                    Err(e) => {
                        // If error is read
                        eprintln!("Error reading from client {}: {}", addr, e);
                        break;
                    }
                }
            }
        });
    }
}
