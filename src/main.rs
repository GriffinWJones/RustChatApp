use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // Async read and write for network sockets
use tokio::net::TcpListener; // Allows sever to listen for TCP connections
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

struct Client {
    stream: Arc<Mutex<TcpStream>>,
    username: String,
}

#[tokio::main] // Initalizes tokio runtine, allows main to be async
async fn main() -> tokio::io::Result<()> {
    // Makes main async, returns a tokio::io::Result<()>
    let listener = TcpListener::bind("127.0.0.1:8080").await?; // Binders a tcp listener to ip and port and waits until binding is complete
    println!("Server listening on port 8080...");

    //let clients: Arc<Mutex<Vec<Arc<Mutex<TcpStream>>>>> = Arc::new(Mutex::new(Vec::new()));
    //Need to keep track of TcpStreams as these are the connections to clients
    //Wrapped in Mutex so that only one client can access a given client's tcp stream at a time
    //Wrapped in ARC (Atomic Reference Counter) to allow multiple owners of a given mutexed tcpstream as each client opperates in a seperate task
    //Wrapped in Vec because I need to keep track of multiple of these objects, one for each connected client
    //Wrapped in Mutex because I need to make sure that the vector is protected as different clients will try to add or remove themselves to the vector, and we cannot have more than one doing this at a time
    //Wrapped in ARC (Atomic Reference Counter) to allow for multiple clients in different tasks to share ownership so they can add and remove themselves
    //Essentially this is a mutex protected list of mutext protected tcp connections

    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        let (mut socket, addr) = listener.accept().await?; // Waits for a client connect and gets the socket and the client's address
        println!("New client connected: {}", addr);

        let username = request_username(&mut socket).await?;
        println!("Client {} identified as: {}", addr, username);

        let socket = Arc::new(Mutex::new(socket)); // Takes the socket (TcpSteam) and wraps it in Mutex and Arc to fit into the clients Vector

        let client = Client {
            stream: Arc::clone(&socket),
            username: username.clone(),
        };

        {
            //This is the start of a new scope
            let clients_clone = Arc::clone(&clients); //Creates a new reference to the data in clients to be used in this scope
            let mut clients_locked = clients_clone.lock().await; //creates clients locked which locks the mutex so that other tasks cannot modify (aquires mutex lock)
            clients_locked.push(client); // This adds the socket to the locked list of clients
        } // This is the end of the new scope
          //Putting the above code in a new scope is needed because when a client exits the code block, the lock is automatically released and other clients can now aquire the lock and add their socket

        let clients_for_task = Arc::clone(&clients); //Creates a reference to data in clients that will be used in the task spawned in the next line

        tokio::spawn(async move {
            // tokio::spawn Creates an async task (light weight thread) for client connection, move takes ownership of surrounding scope
            // This means that main cannot use socket and addr as they are now owned by the task and not main
            // This is also why I can define clients_for_task outside of the task as the task will take ownership of that variable

            let mut buffer = [0; 1024]; // Create buffer for store incoming data from client

            loop {
                let message = {
                    //Creates a new scope
                    let mut socket_locked = socket.lock().await; //lock the socket so it can be read or written to
                    match timeout(Duration::from_millis(10), socket_locked.read(&mut buffer)).await
                    {
                        //Attempt to read the socket into the buffer
                        Ok(Ok(0)) => {
                            //0 read, client disconnected
                            println!("Client {} ({}) disconnected", username, addr);
                            break; // Break connection
                        }
                        Ok(Ok(n)) => {
                            println!("Client {} ({}) has sent a message", username, addr);
                            // Create a formatted message with the username
                            let raw_message =
                                String::from_utf8_lossy(&buffer[..n]).trim().to_string();

                            // Only broadcast if the message is not empty
                            if !raw_message.is_empty() {
                                let formatted_message =
                                    format!("{}: {}\r\n", username, raw_message);
                                Some(formatted_message.into_bytes())
                            } else {
                                None
                            }
                        }
                        Ok(Err(e)) => {
                            // Error reading
                            eprintln!("Error reading from client {} ({}): {}", username, addr, e);
                            break;
                        }
                        Err(_) => {
                            //eprintln!("Timeout error reading from client {} ({})", username, addr);
                            None
                        }
                    }
                }; // Socket lock released here

                if let Some(message) = message {
                    //Will only execute if message contains a Some() value (only works on sucessful read from client)

                    let clients = clients_for_task.lock().await; //Re-lock the clients for task to be able to use in scope

                    for c in clients.iter() {
                        //Loop through all connected clients

                        if Arc::ptr_eq(&c.stream, &socket) {
                            continue; // Skip sending message to the sender (who's socket matches)
                        }

                        let mut client_locked: tokio::sync::MutexGuard<'_, TcpStream> =
                            c.stream.lock().await; //aquire the mutex lock for a given client

                        if let Err(e) = client_locked.write_all(&message).await {
                            //Try to write the message to the current client
                            //write the message to the locked client
                            eprintln!("Failed to write to a client: {}", e);
                            continue;
                        } else {
                            println!("Client {} has recieved a message", c.username);
                        }

                        if let Err(e) = client_locked.flush().await {
                            eprintln!("Failed to flush message to client: {}", e);
                        }
                    } //At the end of each iteration the given client's lock is released
                }
            }
            //This only runs in the task after a client disconnects
            let mut clients_locked = clients_for_task.lock().await; //aquire the mutex lock for the whole vector
            clients_locked.retain(|c| !Arc::ptr_eq(&c.stream, &socket));
            //This filers the clients_locked by only keeping clients who arent the current client
            //This is because this code runs when a client disconnects, so the current client should be taken out of the list of clients
            println!("Client {} removed from clients list", addr);
        });
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
