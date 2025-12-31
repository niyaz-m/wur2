# Whats Up Rust 2 (WUR2)

A robust, asynchronous multi-user chat server built with **Rust**, **Tokio**, and **PostgreSQL**. This project features a custom TCP protocol with built-in authentication, channel management, and real-time messaging.

---

##  Features

* **Asynchronous Architecture**: Powered by `tokio` for high-concurrency handling of TCP connections.
* **Secure Authentication**: User registration and login using `argon2` password hashing.
* 
**Persistent Storage**: PostgreSQL integration via `sqlx` to store users and message history.


* **Channel System**: Support for switching between different chat channels (e.g., Global, Private).
* **Real-time Interaction**:
* **Broadcast**: Message everyone in your current channel.
* **Private Messaging**: Direct messages between users using `/msg`.
* **File Transfer**: Stream files directly to other users using `/send`.


* **Role Management**: Basic "User" and "Mod" roles with privilege checking (e.g., kicking users).

---

##  Tech Stack

* **Language**: Rust
* **Runtime**: [Tokio](https://tokio.rs/) (Async I/O)
* 
**Database**: PostgreSQL 


* **ORM/Query Builder**: [SQLx](https://github.com/launchbadge/sqlx)
* **Security**: Argon2 (Password Hashing)

---

##  Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
* [PostgreSQL](https://www.postgresql.org/download/)
* `sqlx-cli` (optional, for running migrations manually)

---

## ️ Setup

1. **Clone the repository**:
```bash
git clone https://github.com/niyaz-m/wur2.git 
cd wur2

```


2. **Configure Environment**:
Create a `.env` file in the root directory (or update the existing one):


```env
DATABASE_URL=postgres://username:password@localhost/wur2

```


3. **Database Migration**:
The server automatically runs migrations on startup. Ensure your PostgreSQL server is running and the database specified in `.env` exists.
4. **Run the Server**:
```bash
cargo run

```


The server will start on `0.0.0.0:6969`.

---

##  Usage & Commands

Once connected via a TCP client (like `telnet` or `nc`), you can use the following commands:

| Command | Action |
| --- | --- |
| `/msg <user> <msg>` | Send a private message to a specific user. |
| `/join <channel>` | Switch to a different chat channel. |
| `/send <user> <path>` | Send a file from the server's path to a user. |
| `/list` | List all currently connected users. |
| `/channels` | List all active channels. |
| `/profile` | View your username, current channel, and role. |
| `/kick <user>` | Disconnect a user (Moderator only). |
| `/role` | Toggle your role (Demonstration purposes). |
| `/close` | Safely disconnect from the server. |
| `/help` | Show the available command list. |

---

##  Project Structure

* `main.rs`: Entry point, database initialization, and server startup.
* `auth.rs`: Handles the login/registration flow and password verification.
* `server.rs`: Manages the TCP listener and client connection lifecycle.
* `messages.rs`: Contains the command parser and execution logic.
* `users.rs`: Defines the `User` struct and asynchronous communication tasks.
* `db.rs`: Database abstraction layer for user operations.
* `models.rs`: Data structures for database entities.
