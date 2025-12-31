use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum UserMessage {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub channel: String,
    pub role: String,
    pub tx: mpsc::UnboundedSender<UserMessage>,
}

impl User {
    pub async fn from_stream(
        writer: OwnedWriteHalf,
        _reader: &BufReader<&mut OwnedReadHalf>,
        username: &String,
    ) -> io::Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel::<UserMessage>();

        tokio::spawn(Self::writer_task(writer, rx));

        let _ = tx.send(UserMessage::Text("=======================\n".to_string()));
        let _ = tx.send(UserMessage::Text("||  Whats Up Rust 2  ||\n".to_string()));
        let _ = tx.send(UserMessage::Text("=======================\n".to_string()));

        //let username = format!("user_{}", rand::random::<u8>());
        let username = username.trim();
        let channel = "Global".to_string();
        let role = "User".to_string();

        let user = User {
            username: username.to_string(),
            channel,
            role,
            tx,
        };
        Ok(user)
    }

    async fn writer_task(mut writer: OwnedWriteHalf, mut rx: mpsc::UnboundedReceiver<UserMessage>) {
        while let Some(msg) = rx.recv().await {
            let res = match msg {
                UserMessage::Text(s) => writer.write_all(s.as_bytes()).await,
                UserMessage::Binary(b) => writer.write_all(&b).await,
            };
            if res.is_err() {
                break;
            }
        }
    }

    pub async fn send(&self, message: String) -> io::Result<()> {
        self.tx
            .send(UserMessage::Text(message + "\n"))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Client disconnected"))
    }

    pub async fn send_file_stream(&self, mut file: tokio::fs::File) -> io::Result<()> {
        let mut buffer = [0u8; 8192];
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            self.tx
                .send(UserMessage::Binary(buffer[..n].to_vec()))
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "ERROR: receiver closed"))?;
        }
        Ok(())
    }

    pub async fn switch_channel(&mut self, new_channel: String) -> io::Result<()> {
        let old_channel = std::mem::replace(&mut self.channel, new_channel.clone());
        let response = format!("Switched from {} to {}", old_channel, new_channel);
        self.send(response).await
    }

    pub async fn change_role(&mut self) -> io::Result<()> {
        let _ = std::mem::replace(&mut self.role, "Mod".to_string());
        Ok(())
    }

    pub fn get_channel(&self) -> &str {
        &self.channel
    }

    pub fn get_profile(&self) -> String {
        format!(
            "Username: {}\nChannel: {}\nRole: {}\n",
            self.username, self.channel, self.role
        )
    }
}
