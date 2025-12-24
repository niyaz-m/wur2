use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::auth::Auth;
use crate::db::UserDb;
use crate::messages::CommandExecutor;
use crate::users::User;

pub enum ConnectionStatus {
    Continue,
    Close,
}

pub type Users = Arc<Mutex<HashMap<String, User>>>;

pub struct Server;

impl Server {
    pub async fn setup_server(addr: &str, pool: PgPool) -> io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        let users: Users = Arc::new(Mutex::new(HashMap::new()));

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let users = users.clone();
                    let stream = stream;
                    let pool = pool.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, users, pool).await {
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

    async fn handle_client(stream: TcpStream, users: Users, pool: PgPool) -> io::Result<()> {
        let (mut reader, mut writer) = stream.into_split();

        let user_db = UserDb::new(pool);
        let auth = Auth::new(user_db);

        let username = auth.auth(&mut writer, &mut reader).await;
        let mut reader = BufReader::new(reader);

        let username: String = username?;
        let user = User::from_stream(writer, &username).await?;

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

pub async fn start_server(addr: &str, pool: PgPool) -> io::Result<()> {
    Server::setup_server(addr, pool).await
}
