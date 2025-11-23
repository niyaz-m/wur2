use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::messages::broadcast_messages;
use crate::users::User;

pub type Users = Arc<Mutex<HashMap<String, User>>>;

pub async fn start_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let users: Users = Arc::new(Mutex::new(HashMap::new()));

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                //let (socket, _) = listener.accept().await?;
                //println!("New client: {:?}", addr);
                let users = users.clone();
                //let (stream, _) = listener.accept().await.unwrap();
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
//}

async fn handle_client(stream: TcpStream, users: Users) -> io::Result<()> {
    let (reader, stream) = stream.into_split();
    let mut reader = BufReader::new(reader);

    //writer.write_all(b"Hello, World!\n").await?;
    //writer.write_all(b"Enter your username: ").await?;

    let mut username = String::new();
    reader.read_line(&mut username).await?;
    let username = username.trim();

    //let username_msg = format!("Your username is {}\n", username.trim());
    //writer.write_all(username_msg.as_bytes()).await?;

    println!("{} joined the chat", username.to_string());

    let channel = "Global".to_string();

    {
        let mut users = users.lock().await;
        users.insert(
            username.to_string(),
            User::new(username.to_string(), channel.clone(), stream),
        );
        //for (_, User) in users.iter() {
            //println!("User: {:#?}", User);
        //}
    }

    loop {
        let mut msg = String::new();
        reader.read_line(&mut msg).await?;
        broadcast_messages(username, &msg, &users).await?;
    }
}
