use tokio::net::tcp::{OwnedWriteHalf, OwnedReadHalf};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};

pub struct Auth;

impl Auth {
    pub async fn auth(writer: &mut OwnedWriteHalf, reader: &mut OwnedReadHalf) -> String {
        let mut reader = BufReader::new(reader);

        let mut username = String::new();
        if let Err(e) = writer.write(b"Enter username: ").await {
            eprintln!("ERROR: failed to prompt username: {e}");
        }
        if let Err(e) = reader.read_line(&mut username).await {
            eprintln!("ERROR: failed to read username: {e}");
        }

        let mut password = String::new();
        if let Err(e) = writer.write(b"Enter Password: ").await {
            eprintln!("ERROR: failed to prompt password: {e}");
        }
        if let Err(e) = reader.read_line(&mut password).await {
            eprintln!("ERROR: failed to read password: {e}");
        }

        let username = username.trim();
        username.to_string()
    }
}
