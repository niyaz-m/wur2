use tokio::sync::mpsc;

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub channel: String,
    pub tx: mpsc::UnboundedSender<String>,
}

// impl User {
//     pub fn new(username: String, channel: String, tx: mpsc::unbounded_channel::<String>) -> Self {
//         Self { username,
//                channel,
//                tx: mpsc::unbounded_channel::<String>(),
//         }
//     }
// }
