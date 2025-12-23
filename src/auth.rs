use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub struct Auth;

impl Auth {
    pub async fn auth(writer: &mut OwnedWriteHalf, reader: &mut OwnedReadHalf) -> String {
        let mut reader = BufReader::new(reader);
        let mut answer = String::new();
        let _ = writer.write_all(b"Do you an account? (y/n) ").await;
        let _ = reader.read_line(&mut answer).await;
        let answer = answer.trim();
        match answer {
            "y" => {
                let username = Self::credentials(writer, &mut reader).await;
                username.to_string()
            }
            _ => "todo".to_string(),
        }
    }

    async fn credentials(
        writer: &mut OwnedWriteHalf,
        reader: &mut BufReader<&mut OwnedReadHalf>,
    ) -> String {
        let mut username = String::new();
        loop {
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
            let password_hash = Self::hash_password(password.clone()).await;
            println!("INFO: password_hash: {password_hash}");

            let stored_password = "$argon2id$v=19$m=19456,t=2,p=1$QzkgJ0Si+7wjLgliPZ67eA$STP/3almbVBwzOTaYQNCtrazf6/4RENVU1Mt6mI+Zuo";

            let auth = Self::verify_password(password, stored_password).await;
            if auth == true {
                let _ = writer.write_all(b"Your are authenticated\n").await;
                break;
            } else {
                println!("INFO: user entered wrong password");
                let _ = writer.write_all(b"Your are not authenticated\n").await;
                continue;
            }
        }

        let username = username.trim();
        username.to_string()
    }

    async fn hash_password(password: String) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("ERROR: password hashing failed")
            .to_string();
        hash
    }

    async fn verify_password(password: String, stored_hash: &str) -> bool {
        let parsed_hash = PasswordHash::new(stored_hash).expect("ERROR: invalid hash format");

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}
