use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

struct Client {
    stream: Arc<Mutex<TcpStream>>,
    username: String,
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on port 8080...");

    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New client connected: {}", addr);

        let username = request_username(&mut socket).await?;
        println!("Client {} identified as: {}", addr, username);

        let socket = Arc::new(Mutex::new(socket));
        let client = Client {
            stream: Arc::clone(&socket),
            username: username.clone(),
        };

        {
            let clients_clone = Arc::clone(&clients);
            let mut clients_locked = clients_clone.lock().await;
            clients_locked.push(client);
        }

        let clients_clone = Arc::clone(&clients);
        let join_message = format!("*** {} has joined the chat! ***", username);
        broadcast_message_from_server(&clients_clone, &join_message).await;

        let clients_for_task = Arc::clone(&clients);

        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            loop {
                let message = {
                    let mut socket_locked = socket.lock().await;
                    match timeout(Duration::from_millis(10), socket_locked.read(&mut buffer)).await
                    {
                        Ok(Ok(0)) => {
                            println!("Client {} ({}) disconnected", username, addr);
                            break;
                        }
                        Ok(Ok(n)) => {
                            println!("Client {} ({}) has sent a message", username, addr);

                            let raw_message =
                                String::from_utf8_lossy(&buffer[..n]).trim().to_string();

                            if raw_message == "EXIT" {
                                println!("Client {} ({}) disconnected", username, addr);
                                break;
                            }

                            if !raw_message.is_empty() {
                                let formatted_message =
                                    format!("{}: {}\r\n", username, raw_message);
                                Some(formatted_message.into_bytes())
                            } else {
                                None
                            }
                        }
                        Ok(Err(e)) => {
                            eprintln!("Error reading from client {} ({}): {}", username, addr, e);
                            break;
                        }
                        Err(_) => None,
                    }
                };

                if let Some(message) = message {
                    let clients = clients_for_task.lock().await;

                    for c in clients.iter() {
                        if Arc::ptr_eq(&c.stream, &socket) {
                            continue;
                        }

                        let mut client_locked: tokio::sync::MutexGuard<'_, TcpStream> =
                            c.stream.lock().await;

                        if let Err(e) = client_locked.write_all(&message).await {
                            eprintln!("Failed to write to a client: {}", e);
                            continue;
                        } else {
                            println!("Client {} has recieved a message", c.username);
                        }

                        if let Err(e) = client_locked.flush().await {
                            eprintln!("Failed to flush message to client: {}", e);
                        }
                    }
                }
            }
            let leave_message = format!("*** {} has left the chat. ***", username);
            broadcast_message_from_server(&clients_for_task, &leave_message).await;
            println!("Client {} removed from clients list", addr);

            let mut clients_locked = clients_for_task.lock().await;
            clients_locked.retain(|c| !Arc::ptr_eq(&c.stream, &socket));
        });
    }
}

async fn broadcast_message_from_server(clients: &Arc<Mutex<Vec<Client>>>, message: &str) {
    let formatted_message = format!("Server: {}\r\n", message);

    let clients_locked = clients.lock().await;

    for c in clients_locked.iter() {
        let mut client_locked = c.stream.lock().await;

        if let Err(e) = client_locked.write_all(formatted_message.as_bytes()).await {
            eprintln!("Failed to write to client {}: {}", c.username, e);
        }

        if let Err(e) = client_locked.flush().await {
            eprintln!("Failed to flush message to client {}: {}", c.username, e);
        }
    }
}

async fn request_username(socket: &mut TcpStream) -> tokio::io::Result<String> {
    let prompt = "Enter your username: ".as_bytes();
    socket.write_all(prompt).await?;
    socket.flush().await?;

    let mut buffer = [0; 1024];
    let n = socket.read(&mut buffer).await?;

    let username = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

    Ok(username)
}
