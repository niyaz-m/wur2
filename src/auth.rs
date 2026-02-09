use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Result};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::db::UserDb;

pub struct Auth {
    user_db: UserDb,
}

impl Auth {
    pub fn new(user_db: UserDb) -> Self {
        Self { user_db }
    }

    pub async fn auth(
        &self,
        writer: &mut OwnedWriteHalf,
        reader: &mut OwnedReadHalf,
    ) -> Result<String> {
        loop {
            let mut answer = String::new();
            writer
                .write_all(b"Do you have an account? (y/n) \n")
                .await?;
            let mut reader = BufReader::new(&mut *reader);
            reader.read_line(&mut answer).await?;
            let answer = answer.trim();
            match answer {
                "y" => {
                    return self.login(writer, &mut reader).await;
                }
                "n" => {
                    return self.register(writer, &mut reader).await;
                }
                _ => {
                    continue;
                }
            }
        }
    }

    async fn register(
        &self,
        writer: &mut OwnedWriteHalf,
        reader: &mut BufReader<&mut OwnedReadHalf>,
    ) -> Result<String> {
        loop {
            let Ok((username, password)) = Self::credentials(writer, reader).await else {
                todo!()
            };
            let username = username.trim();
            let password = password.trim();
            if !password.is_empty() && !username.is_empty() {
                let password_hash = Self::hash_password(password.to_string().clone()).await;
                println!("INFO: password hash: {password_hash}");
                match self
                    .user_db
                    .create_user(username.to_string(), password_hash.as_str())
                    .await
                {
                    Ok(username) => println!("INFO: {:#?} created", username),
                    Err(e) => {
                        println!("ERROR: failed to create user {username}: {}", e);
                        let response = format!("{username} is taken. Chosse another username.\n");
                        Self::write_line(writer, response.as_str()).await?;
                        continue;
                    }
                }
                Self::list_users(&self.user_db).await?;
                break Ok(username.to_string());
            }
        }
    }

    async fn login(
        &self,
        writer: &mut OwnedWriteHalf,
        reader: &mut BufReader<&mut OwnedReadHalf>,
    ) -> Result<String> {
        loop {
            let Ok((username, password)) = Self::credentials(writer, reader).await else {
                panic!("ERROR: failed to get crdentials");
            };
            let user = self.user_db.find_by_username(username.as_str()).await;
            match user {
                Ok(Some(user)) => {
                    let is_valid = Self::verify_password(password, &user.password_hash).await;
                    if is_valid {
                        let response = format!("Welcome back {username}!\n");
                        Self::write_line(writer, response.as_str()).await?;
                        let username = username.trim();
                        println!("INFO: {username} logged in");
                        Self::list_users(&self.user_db).await?;
                        break Ok(username.to_string());
                    } else {
                        Self::write_line(writer, "You entered wrong password").await?;
                        println!("WARN: user entered wrong password");
                        continue;
                    }
                }
                Ok(None) => {
                    Self::write_line(writer, "user doesn't exist").await?;
                    println!("WARN: user doesn't exist");
                }
                Err(e) => {
                    println!("ERROR: failed to login: {}", e);
                    continue;
                }
            }
        }
    }

    async fn credentials(
        writer: &mut OwnedWriteHalf,
        reader: &mut BufReader<&mut OwnedReadHalf>,
    ) -> Result<(String, String)> {
        let username = Self::read_input(writer, reader, "Enter username \n").await?;
        let password = Self::read_input(writer, reader, "Enter password \n").await?;
        Ok((username, password))
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

    pub async fn list_users(user_db: &UserDb) -> Result<()> {
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

    async fn write_line(writer: &mut OwnedWriteHalf, message: &str) -> Result<()> {
        writer
            .write_all(format!("{}\n", message).as_bytes())
            .await?;
        writer.flush().await?;
        Ok(())
    }

    async fn read_input(
        writer: &mut OwnedWriteHalf,
        reader: &mut BufReader<&mut OwnedReadHalf>,
        prompt: &str,
    ) -> Result<String> {
        Self::prompt_user(writer, prompt).await?;
        let mut input = String::new();
        reader.read_line(&mut input).await?;
        Ok(input.trim().to_string())
    }

    async fn prompt_user(writer: &mut OwnedWriteHalf, prompt: &str) -> Result<()> {
        writer.write_all(prompt.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }
}
