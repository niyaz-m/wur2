use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::messages::CommandExecutor;
use crate::users::User;

pub enum ConnectionStatus {
    Continue,
    Close,
}

pub type Users = Arc<Mutex<HashMap<String, User>>>;

pub struct Server;

impl Server {
    pub async fn start_server(addr: &str) -> io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        let users: Users = Arc::new(Mutex::new(HashMap::new()));

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let users = users.clone();
                    let stream = stream;
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, users).await {
                            eprintln!("ERROR: failed to handle client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    println!("ERROR: Couldn't get client: {:?}", e);
                    break Ok(());
                }
            }
        }
    }

    async fn handle_client(stream: TcpStream, users: Users) -> io::Result<()> {
        let (reader, writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let user = User::from_stream(writer).await?;

        {
            let mut users_guard = users.lock().await;
            users_guard.insert(user.username.clone(), user.clone());
        }

        println!("INFO: {} connected", user.username);

        let mut buffer = String::new();

        while reader.read_line(&mut buffer).await? > 0 {
            let command = buffer.trim().to_string();
            match CommandExecutor::execute(user.username.clone(), command, users.clone()).await {
                Ok(ConnectionStatus::Continue) => {}
                Ok(ConnectionStatus::Close) => break,
                Err(e) => return Err(e),
            }
            buffer.clear();
        }

        {
            let mut users_guard = users.lock().await;
            users_guard.remove(&user.username);
        }

        println!("INFO: {} disconnected", user.username);

        Ok(())
    }
}

pub async fn start_server(addr: &str) -> io::Result<()> {
    Server::start_server(addr).await
}
