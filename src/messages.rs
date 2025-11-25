use tokio::io;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;


use crate::users::User;

pub type Users = Arc<Mutex<HashMap<String, User>>>;

pub async fn broadcast_messages(
    sender_name: &str,
    msg: &str,
    users: &Users,
) -> io::Result<()> {
    let users = users.lock().await;
    
    for (name, user) in users.iter() {
        if name != &sender_name {
            let _ = user.tx.send(format!("[{0}] {sender_name}: {msg}\n", user.channel));
        }
    }
    
    Ok(())
}

