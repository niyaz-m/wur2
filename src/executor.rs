use tokio::io;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::users::User;
use crate::messages::broadcast_messages;

pub type Users = Arc<Mutex<HashMap<String, User>>>;

pub async fn executor(
    username: String,
    msg: String,
    users: Users
) -> io::Result<()> {
    match msg.as_str() {
        "/list" => {
            let mut users = users.lock().await;
            let list = users.keys().cloned().collect::<Vec<_>>()
                .join(", ");
            let response = format!("Connected users: {}\n", list);
            if let Some(user) = users.get_mut(&username) {
                let _ = user.tx.send(response);
            }
        }

        _ => broadcast_messages(&username, &msg, &users).await?,
            
    }
    Ok(())
}
