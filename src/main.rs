mod messages;
mod server;
mod users;
mod db;

use crate::server::start_server;
use crate::db::db_connection;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    db_connection().await;

    println!("INFO: server starting on port :6969");
    if let Err(e) = start_server("127.0.0.1:6969").await {
        eprintln!("ERROR: failed to start server: {}", e);
    }
    Ok(())
}
