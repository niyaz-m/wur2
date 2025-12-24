use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::db::UserDb;

pub struct Auth {
    user_db: UserDb,
}

impl Auth {
    pub fn new(user_db: UserDb) -> Self {
        Self { user_db }
    }

    pub async fn auth(&self, writer: &mut OwnedWriteHalf, reader: &mut OwnedReadHalf) -> String {
        let mut reader = BufReader::new(reader);
        let mut answer = String::new();
        let _ = writer.write_all(b"Do you an account? (y/n) ").await;
        let _ = reader.read_line(&mut answer).await;
        let answer = answer.trim();
        match answer {
            "y" => {
                let username = Self::login(writer, &mut reader).await;
                username.to_string()
            }
            "n" => {
                let username = self.register(writer, &mut reader).await;
                username.to_string()
            }
            _ => "Do you an account? (y/n) ".to_string(),
        }
    }

    async fn register(
        &self,
        writer: &mut OwnedWriteHalf,
        reader: &mut BufReader<&mut OwnedReadHalf>,
    ) -> String {
        loop {
            let (username, password) = Self::credentials(writer, reader).await;
            let username = username.trim();
            let password = password.trim();
            if !password.is_empty() && !username.is_empty() {
                let password_hash = Self::hash_password(password.to_string().clone()).await;
                println!("INFO: password hash: {password_hash}");
                match self.user_db.create_user(username.to_string(), password_hash.as_str()).await {
                    Ok(username) => println!("INFO: {:?} created", username),
                    Err(e) => println!("ERROR: failed to create user {username}: {}", e),
                }
                let _ = Self::list_users(&self.user_db).await;
                break username.to_string();
            }
        }
    }

    async fn login(
        writer: &mut OwnedWriteHalf,
        reader: &mut BufReader<&mut OwnedReadHalf>,
    ) -> String {
        loop {
            let (username, password) = Self::credentials(writer, reader).await;

            let password_hash = Self::hash_password(password.clone()).await;
            println!("INFO: password_hash: {password_hash}");
            let stored_password = "$argon2id$v=19$m=19456,t=2,p=1$QzkgJ0Si+7wjLgliPZ67eA$STP/3almbVBwzOTaYQNCtrazf6/4RENVU1Mt6mI+Zuo";

            let auth = Self::verify_password(password, stored_password).await;
            if auth == true {
                let _ = writer.write_all(b"Your are authenticated\n").await;
                let username = username.trim();
                println!("INFO: {username} logged in");
                break username.to_string();
            } else {
                println!("INFO: user entered wrong password");
                let _ = writer.write_all(b"Your are not authenticated\n").await;
                continue;
            }
        }
    }

    async fn credentials(
        writer: &mut OwnedWriteHalf,
        reader: &mut BufReader<&mut OwnedReadHalf>,
    ) -> (String, String) {
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
        (username, password)
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

    pub async fn list_users(user_db: &UserDb) -> Result<(), Box<dyn std::error::Error>> {
        match user_db.get_all_users().await {
            Ok(users) => {
                if users.is_empty() {
                    println!("INFO: no users found in the database.");
                } else {
                    println!("=== Users in Database ===");
                    println!("{:<5} {:<20} {:<25}", "ID", "Username", "Created At");
                    println!("{}", "-".repeat(55));

                    for user in users.clone() {
                        let created_at = user.created_at.format("%Y-%m-%d %H:%M:%S");
                        println!("{:<5} {:<20} {:<25}", user.id, user.username, created_at);
                    }
                    println!("Total: {} users", users.len());
                }
            }
            Err(e) => println!("ERROR: fetching users: {}", e),
        }

        Ok(())
    }
    async fn verify_password(password: String, stored_hash: &str) -> bool {
        let parsed_hash = PasswordHash::new(stored_hash).expect("ERROR: invalid hash format");

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}
