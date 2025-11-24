use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;

use crate::messages::broadcast_messages;
use crate::users::User;

pub type Users = Arc<Mutex<HashMap<String, User>>>;

pub async fn executor(username: String,
                      msg: String,
                      users: Users,
                      mut stream: OwnedWriteHalf 
                      ) -> io::Result<()> {
    let msg = msg.trim();
    match msg {
        "/list" => {
            let users = users.lock().await;
            let list = users.keys().cloned().collect::<Vec<_>>().join(", ");
            let response = format!("Connected users: {}\n", list);
            //let mut writer = User::stream.lock().await;
            stream.write_all(response.as_bytes()).await?;
        }

        _ => {
            let _ = broadcast_messages(username.as_str(), &msg, &users).await;
        }
    }
    Ok(())
}
