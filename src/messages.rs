use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::io;
use tokio::sync::Mutex;

use crate::users::User;

pub type Users = Arc<Mutex<HashMap<String, User>>>;

#[derive(Debug)]
pub enum Command {
    PrivateMessage { target: String, message: String },
    JoinChannel(String),
    ListUsers,
    ListChannels,
    Broadcast(String),
    Unknown,
}

impl Command {
    pub fn parse(input: String) -> Self {
        let parts: Vec<&str> = input.splitn(3, ' ').collect();

        match parts.as_slice() {
            ["/msg", target, message] => Command::PrivateMessage {
                target: target.to_string(),
                message: message.to_string(),
            },
            ["/join", channel] => Command::JoinChannel(channel.to_string()),
            ["/list"] => Command::ListUsers,
            ["/channels"] => Command::ListChannels,
            [""] => Command::Unknown,
            _ if input.starts_with('/') => Command::Unknown,
            _ => Command::Broadcast(input),
        }
    }
}

pub struct CommandExecutor;

impl CommandExecutor {
    pub async fn execute(username: String, input: String, users: Users) -> io::Result<()> {
        let command = Command::parse(input);

        match command {
            Command::PrivateMessage { target, message } => {
                Self::send_private_message(username, target, message, users).await
            }
            Command::JoinChannel(channel) => Self::join_channel(username, channel, users).await,
            Command::ListUsers => Self::list_users(username, users).await,
            Command::ListChannels => Self::list_channels(username, users).await,
            Command::Broadcast(message) => Self::broadcast_messages(username, message, users).await,
            Command::Unknown => Self::send_unknown_command(username, users).await,
        }
    }

    async fn broadcast_messages(sender_name: String, msg: String, users: Users) -> io::Result<()> {
        let users_guard = users.lock().await;

        let sender_channel = users_guard
            .get(&sender_name)
            .map(|user| user.get_channel())
            .unwrap_or("general");

        println!(
            "DEBUG: {} broadcasting to channel: {}",
            sender_name, sender_channel
        );

        for (name, user) in users_guard.iter() {
            if name != &sender_name && user.get_channel() == sender_channel {
                let final_msg = format!("[{}] {sender_name}: {msg}\n", sender_channel);
                let _ = user.tx.send(final_msg.to_string());
            }
        }
        Ok(())
    }

    async fn join_channel(username: String, channel: String, users: Users) -> io::Result<()> {
        let mut users_guard = users.lock().await;

        if let Some(user) = users_guard.get_mut(&username) {
            user.switch_channel(channel).await?;
        }
        Ok(())
    }
    async fn list_users(username: String, users: Users) -> io::Result<()> {
        let mut users = users.lock().await;
        let list = users.keys().cloned().collect::<Vec<_>>().join(", ");
        let response = format!("Connected users: {}\n", list);
        if let Some(user) = users.get_mut(&username) {
            let _ = user.tx.send(response);
        }
        Ok(())
    }

    async fn send_private_message(
        username: String,
        target_name: String,
        msg: String,
        users: Users,
    ) -> io::Result<()> {
        let mut users = users.lock().await;

        if let Some(_target_user) = users.values().find(|u| u.username == target_name) {
            let response = format!("[DM] {}: {}\n", username, msg);
            if let Some(user) = users.get_mut(&target_name) {
                let _ = user.tx.send(response);
            }
        } else {
            let error = format!("User {} not found.\n", target_name);
            if let Some(user) = users.get_mut(&username) {
                let _ = user.tx.send(error);
            }
        }
        Ok(())
    }

    async fn list_channels(username: String, users: Users) -> io::Result<()> {
        let users_guard = users.lock().await;
        let channels: HashSet<String> = users_guard
            .values()
            .map(|user| user.get_channel().to_string())
            .collect();

        let channel_list: Vec<String> = channels.into_iter().collect();

        if let Some(user) = users_guard.get(&username) {
            user.send(format!("Active channels: {}", channel_list.join(", ")))
                .await?;
        }
        Ok(())
    }

    async fn send_unknown_command(username: String, users: Users) -> io::Result<()> {
        let mut users_guard = users.lock().await;

        if let Some(user) = users_guard.get_mut(&username) {
            let help = r#"
            Available commands:
            /msg <user> <message> - Send private message
            /join <channel> - Switch channels
            /list - List online users
            /channels - List active channels
            "#;
            user.send(help.trim().to_string()).await?;
        }
        Ok(())
    }
}
