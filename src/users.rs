use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub channel: String,
    pub stream: Arc<Mutex<OwnedWriteHalf>>,
}

impl User {
    pub fn new(username: String, channel: String, stream: OwnedWriteHalf) -> Self {
        Self { username,
               channel,
               stream: Arc::new(Mutex::new(stream)),
        }
    }
}
