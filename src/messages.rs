use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::io;
use tokio::sync::Mutex;

use crate::server::ConnectionStatus;
use crate::users::User;

pub type Users = Arc<Mutex<HashMap<String, User>>>;

#[derive(Debug)]
pub enum Command {
    PrivateMessage { target: String, message: String },
    JoinChannel(String),
    ListUsers,
    ListChannels,
    CloseConnection,
    KickUser(String),
    Broadcast(String),
    ProfileView,
    ChangeRole,
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
            ["/kick", target] => Command::KickUser(target.to_string()),
            ["/join", channel] => Command::JoinChannel(channel.to_string()),
            ["/list"] => Command::ListUsers,
            ["/channels"] => Command::ListChannels,
            ["/profile"] => Command::ProfileView,
            ["/role"] => Command::ChangeRole,
            ["/close"] => Command::CloseConnection,
            ["/help"] => Command::Unknown,
            [""] => Command::Unknown,

            _ if input.starts_with('/') => Command::Unknown,
            _ => Command::Broadcast(input),
        }
    }
}

pub struct CommandExecutor;

impl CommandExecutor {
    pub async fn execute(
        username: String,
        input: String,
        users: Users,
    ) -> io::Result<ConnectionStatus> {
        let command = Command::parse(input);

        match command {
            Command::PrivateMessage { target, message } => {
                Self::send_private_message(username, target, message, users).await
            }
            Command::KickUser(target) => Self::kick_user(username, target, users).await,
            Command::JoinChannel(channel) => Self::join_channel(username, channel, users).await,
            Command::ListUsers => Self::list_users(username, users).await,
            Command::ListChannels => Self::list_channels(username, users).await,
            Command::Broadcast(message) => Self::broadcast_messages(username, message, users).await,
            Command::ProfileView => Self::profile_view(username, users).await,
            Command::ChangeRole => Self::change_role(username, users).await,
            Command::CloseConnection => Self::close_connection(username, users).await,
            Command::Unknown => Self::send_unknown_command(username, users).await,
        }
    }

    async fn broadcast_messages(
        sender_name: String,
        msg: String,
        users: Users,
    ) -> io::Result<ConnectionStatus> {
        let users_guard = users.lock().await;

        let sender_channel = users_guard
            .get(&sender_name)
            .map(|user| user.get_channel())
            .unwrap_or("general");

        println!(
            "INFO: {} broadcasing to channel: {}",
            sender_name, sender_channel
        );

        for (name, user) in users_guard.iter() {
            if name != &sender_name && user.get_channel() == sender_channel {
                let final_msg = format!("[{}] {sender_name}: {msg}", sender_channel);
                user.send(final_msg).await?;
            }
        }
        Ok(ConnectionStatus::Continue)
    }

    async fn join_channel(
        username: String,
        channel: String,
        users: Users,
    ) -> io::Result<ConnectionStatus> {
        let mut users_guard = users.lock().await;
        if let Some(user) = users_guard.get_mut(&username) {
            user.switch_channel(channel).await?;
        }
        Ok(ConnectionStatus::Continue)
    }

    async fn change_role(username: String, users: Users) -> io::Result<ConnectionStatus> {
        let mut users_guard = users.lock().await;
        if let Some(user) = users_guard.get_mut(&username) {
            user.change_role().await?;
            let response = format!("You changed your role");
            Self::send_message(username, users_guard, response).await?;
        }
        Ok(ConnectionStatus::Continue)
    }

    async fn profile_view(username: String, users: Users) -> io::Result<ConnectionStatus> {
        let mut users_guard = users.lock().await;
        if let Some(user) = users_guard.get_mut(&username) {
            let profile = user.get_profile();
            Self::send_message(username, users_guard, profile).await?;
        }
        Ok(ConnectionStatus::Continue)
    }

    async fn list_users(username: String, users: Users) -> io::Result<ConnectionStatus> {
        let users_guard = users.lock().await;
        let list = users_guard.keys().cloned().collect::<Vec<_>>().join(", ");
        let response = format!("Connected users: {}\n", list);
        Self::send_message(username, users_guard, response.to_string()).await?;
        Ok(ConnectionStatus::Continue)
    }

    async fn kick_user(
        kicker: String,
        target: String,
        users: Users,
    ) -> io::Result<ConnectionStatus> {
        let mut users_guard = users.lock().await;

        if let Some(user) = users_guard.values().find(|u| u.role == "User") {
            let response = format!("You don't have the privileges to kick users...");
            user.send(response.to_string()).await?;
            return Ok(ConnectionStatus::Continue);
        }

        if kicker == target {
            let response = format!("You cannot kick yourself...");
            Self::send_message(kicker, users_guard, response.to_string()).await?;
            return Ok(ConnectionStatus::Continue);
        }

        if !users_guard.contains_key(&target) {
            let response = format!("{target} not found...");
            Self::send_message(kicker, users_guard, response.to_string()).await?;
            return Ok(ConnectionStatus::Continue);
        }

        if let Some(user) = users_guard.remove(&target) {
            let response = format!("You have been kicked out of the server...");
            user.send(response).await?;
        }
        return Ok(ConnectionStatus::Continue);
    }

    async fn send_private_message(
        username: String,
        target_name: String,
        msg: String,
        users: Users,
    ) -> io::Result<ConnectionStatus> {
        let users_guard = users.lock().await;
        if let Some(_target_user) = users_guard.values().find(|u| u.username == target_name) {
            let response = format!("[DM] {}: {}", username, msg);
            Self::send_message(target_name, users_guard, response.to_string()).await?;
        } else {
            let error = format!("User {} not found.", target_name);
            Self::send_message(username, users_guard, error.to_string()).await?;
        }
        Ok(ConnectionStatus::Continue)
    }

    async fn list_channels(username: String, users: Users) -> io::Result<ConnectionStatus> {
        let users_guard = users.lock().await;
        let channels: HashSet<String> = users_guard
            .values()
            .map(|user| user.get_channel().to_string())
            .collect();

        let channel_list: Vec<String> = channels.into_iter().collect();
        let response = format!("Active channels: {}", channel_list.join(", "));
        Self::send_message(username, users_guard, response.to_string()).await?;
        Ok(ConnectionStatus::Continue)
    }

    async fn close_connection(username: String, users: Users) -> io::Result<ConnectionStatus> {
        //let users_guard = users.lock().await;
        let response = format!("GOODBYE!");
        //Self::send_message(username, users_guard, response.to_string()).await?;
        Self::c_send_message(username, users, response.to_string()).await?;
        return Ok(ConnectionStatus::Close);
    }

    pub async fn c_send_message(
        target: String,
        users: Arc<Mutex<HashMap<String, User>>>,
        message: String,
    ) -> io::Result<()> {
        let sender = {
            let users = users.lock().await;
            users.get(&target).cloned()
        }; // lock DROPPED

        if let Some(sender) = sender {
            sender.send(message).await?;
        }

        Ok(())
    }

    pub async fn send_message(
        target: String,
        mut users_guard: tokio::sync::MutexGuard<'_, HashMap<String, User>>,
        message: String,
    ) -> io::Result<()> {
        if let Some(user) = users_guard.get_mut(&target) {
            user.send(message).await?;
        }
        Ok(())
    }

    async fn send_unknown_command(username: String, users: Users) -> io::Result<ConnectionStatus> {
        let mut users_guard = users.lock().await;

        if let Some(user) = users_guard.get_mut(&username) {
            let help = r#"
            Available commands:
            /msg <user> <message> - Send private message
            /join <channel> - Switch channels
            /list - List online users
            /channels - List active channels
            /profile - Show your profile
            /close - Close the connection
            /help = To show this message
            "#;
            user.send(help.trim().to_string()).await?;
        }
        Ok(ConnectionStatus::Continue)
    }
}
