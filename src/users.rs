use tokio::io::{self, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub channel: String,
    pub tx: mpsc::UnboundedSender<String>,
}

impl User {
    pub async fn from_stream(stream: OwnedWriteHalf) -> io::Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel::<String>();

        tokio::spawn(Self::writer_task(stream, rx));

        //let _ = tx.send("Enter username: ".to_string());
        //let mut username = String::new();
        //reader.read_line(&mut username).await?;
        //let username = username.trim();
        //let username_msg = format!("Your username is {}\n", username.trim());
        //let _ = tx.send(username_msg.to_string());

        let username = format!("user_{}", rand::random::<u16>());
        let channel = "Global".to_string();

        let user = User {
            username,
            channel,
            tx,
        };

        user.send("=======================".to_string()).await?;
        user.send("||  Whats Up Rust 2  ||".to_string()).await?;
        user.send("=======================".to_string()).await?;

        Ok(user)
    }

    async fn writer_task(mut writer: OwnedWriteHalf, mut rx: mpsc::UnboundedReceiver<String>) {
        while let Some(msg) = rx.recv().await {
            if let Err(_) = writer.write_all(msg.as_bytes()).await {
                break;
            }
        }
    }

    pub async fn send(&self, message: String) -> io::Result<()> {
        self.tx
            .send(message + "\n")
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Client disconnected"))
    }

    pub async fn switch_channel(&mut self, new_channel: String) -> io::Result<()> { 
        let old_channel = std::mem::replace(&mut self.channel, new_channel.clone());
        self.send(format!("Switched from {} to {}", old_channel, new_channel))
            .await
    }

    pub fn get_channel(&self) -> &str {
        &self.channel
    }
}
