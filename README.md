
---

# Whats Up Rust 2 (WUR2)

Whats Up Rust 2 (WUR2) is a robust, asynchronous, multi-user chat system written in **Rust**, featuring a high-performance TCP server and multiple clients, including a modern **GUI client built with Slint** and classic terminal clients like `telnet` or `nc`.

The project is intentionally split into **server** and **client binaries**, following a real-world architecture. 

---

## Features

### Server

* Asynchronous architecture powered by `tokio`
* Custom line-based TCP protocol
* Persistent storage using PostgreSQL via `sqlx`
* Secure authentication with Argon2 password hashing
* Channel-based chat system
* Basic role management (`User`, `Mod`)

### Messaging

* Channel-wide broadcast messages
* Private messaging using `/msg`
* File transfer using `/send`
* Real-time user and channel listing

### Clients

* Desktop GUI client built using Slint
* Fully compatible with `telnet` and `nc`
* Multiple clients can connect simultaneously
* GUI and terminal users can chat together

---

## Architecture Overview

WUR2 is a multi-binary Cargo project:

```
wur2/
├── server/        # TCP chat server
├── ui/            # Slint-based GUI client
├── shared/        # Shared protocol and models (if applicable)
├── Cargo.toml
└── README.md
```

* The server runs independently and must be started first
* Clients connect over TCP and contain no server logic
* The GUI is a pure client, not a wrapper around the server

---

## Tech Stack

* Language: Rust
* Async Runtime: Tokio
* Database: PostgreSQL
* Database Layer: SQLx
* Security: Argon2
* GUI: Slint
* Protocol: Custom TCP

---

## Prerequisites

* Rust (latest stable)
* PostgreSQL
* Optional: `sqlx-cli`
* A running PostgreSQL instance

---

## Setup

### 1. Clone the repository

```bash
git clone https://github.com/niyaz-m/wur2.git
cd wur2
```

### 2. Configure environment variables

Create a `.env` file in the project root:

```env
DATABASE_URL=postgres://username:password@localhost/wur2
```

Ensure the database exists before starting the server.

### 3. Run the server

```bash
cargo run --bin server
```

The server listens on:

```
0.0.0.0:6969
```

Database migrations are applied automatically on startup.

---

## Running Clients

### GUI Client (Slint)

```bash
cargo run --bin client
```

* Connects to the server over TCP
* Displays chat history and live messages
* Multiple GUI instances can run simultaneously

### Terminal Client

Using telnet:

```bash
telnet 127.0.0.1 6969
```

Or using netcat:

```bash
nc 127.0.0.1 6969
```

If you are going to chat over different devices replace 127.0.0.1 with your ip address.

GUI and terminal clients operate on the same server and can interact in real time.

---

## Usage and Commands

| Command                 | Description                  |
| ----------------------- | ---------------------------- |
| `/msg <user> <message>` | Send a private message       |
| `/join <channel>`       | Switch to another channel    |
| `/send <user> <path>`   | Send a file                  |
| `/list`                 | List connected users         |
| `/channels`             | List active channels         |
| `/profile`              | View your profile            |
| `/kick <user>`          | Kick a user (Moderator only) |
| `/role`                 | Toggle role (demo)           |
| `/close`                | Disconnect safely            |
| `/help`                 | Show available commands      |

Commands behave identically across GUI and terminal clients.

---

## Server Code Structure

* `main.rs` – Application entry point
* `server.rs` – TCP listener and connection lifecycle
* `auth.rs` – Login, registration, password verification
* `messages.rs` – Command parsing and execution
* `users.rs` – User state and async communication
* `db.rs` – PostgreSQL abstraction layer

---

## Project Status

* Server: Stable
* GUI Client: Active development
* Protocol: Evolving
* File transfer: Functional but not optimized

---
