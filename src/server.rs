use tokio::io::{self, AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use std::collections::HashMap;
use std::sync::Arc;

use crate::users::User;
//use crate::messages::broadcast_messages;
use crate::executor::executor;

pub type Users = Arc<Mutex<HashMap<String, User>>>;

pub async fn start_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let users: Users = Arc::new(Mutex::new(HashMap::new()));

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let users = users.clone();
                let stream = stream;
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, users).await {
                        eprintln!("ERROR: failed to handle client: {}", e);
                    }
                });
                             
            }
            Err(e) => {
                println!("Couldn't get client: {:?}", e);
                break Ok(());
            }
        }
    }
}

async fn handle_client(stream: TcpStream, users: Users) -> io::Result<()> {
    let (reader, stream) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    tokio::spawn(spawn_writer(stream, rx));

    let _ = tx.send("Enter username: ".to_string());
    let mut username = String::new();
    reader.read_line(&mut username).await?;
    let username = username.trim();
    let username_msg = format!("Your username is {}\n", username.trim());
    let _ = tx.send(username_msg.to_string());

    println!("{} connected", username.to_string());

    let channel = "Global".to_string();
    
    {
        let mut users = users.lock().await;
        users.insert(
            username.to_string(),
            User {
                username: username.to_string(),
                channel: "Global".to_string(),
                tx: tx.clone(),
            },
        );
    }

    let mut msg = String::new();
    
    loop {
        msg.clear();
        let message = reader.read_line(&mut msg).await?;
        if message == 0 {
            break;
        }
        
        let msg = msg.trim();
        executor(username.to_string(), msg.to_string(), users.clone()).await?;
        //broadcast_messages(username, &msg, &users).await?;
    }

    {
        let mut users = users.lock().await;
        users.remove(username);
    }
    println!("{username} disconnected");

    Ok(())
} 

pub async fn spawn_writer(
    mut writer: tokio::net::tcp::OwnedWriteHalf,
    mut rx: mpsc::UnboundedReceiver<String>,
) {
    while let Some(msg) = rx.recv().await {
        if writer.write_all(msg.as_bytes()).await.is_err() {
            break; // client disconnected
        }
    }
}
