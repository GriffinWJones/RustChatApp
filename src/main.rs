// use std::collections::VecDeque;
// use std::sync::Arc;
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::{TcpListener, TcpStream};
// use tokio::sync::Mutex;

// #[tokio::main]

// async fn main() -> tokio::io::Result<()> {
//     let clients: Arc<Mutex<VecDeque<TcpStream>>> = Arc::new(Mutex::new(VecDeque::new()));

//     let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").await?;
//     print!("Server listening on port 7878...");

//     loop {
//         let (mut socket, addr) = listener.accept().await?;
//         println!("New client connected: {}", addr);

//         let clients = Arc::clone(&clients);
//         tokio::spawn(async move {
//             let mut socket = socket;
//             let mut buffer = [0; 1024];

//             {
//                 let mut clients = clients.lock().await;
//                 clients.push_back(socket);
//             }
//             loop {
//                 match socket.read(&mut buffer).await {
//                     Ok(0) => {
//                         println!("Client {} disconnected", addr);
//                         break;
//                     }
//                     Ok(n) => {
//                         if let Err(e) = socket.write_all(&buffer[..n]).await {
//                             eprintln!("Failed to write to client {}: {}", addr, e);
//                             break;
//                         }
//                     }
//                     Err(e) => {
//                         eprintln!("Error reading from client {}: {}", addr, e);
//                         break;
//                     }
//                 }
//             }
//             {
//                 let mut clients = clients.lock().await;
//                 clients.retain(|client| client.peer_addr().unwrap() != addr);
//             }
//         });
//     }
// }
