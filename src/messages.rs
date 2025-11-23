use tokio::io::{self, AsyncWriteExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

use crate::users::User;

pub type Users = Arc<Mutex<HashMap<String, User>>>;

pub async fn broadcast_messages(
    sender_name: &str,
    message: &str,
    users: &Users,
) -> io::Result<()> {
    let mut users = users.lock().await;
    for user in users.values_mut() {
        //if user.stream.peer_addr()? != sender_stream.peer_addr()? {
            let mut writer = user.stream.lock().await;
            let final_message = format!("{}: {}\n", sender_name.trim(), message.trim());
            let _ = writer.write_all(final_message.as_bytes()).await?;
        //}
    }
    Ok(())
}

