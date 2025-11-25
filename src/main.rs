mod messages;
mod executor;
mod server;
mod users;

use crate::server::start_server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Server starting on port :6969");
    if let Err(e) = start_server("127.0.0.1:6969").await {
        eprintln!("ERROR: failed to start server: {}", e);
    }
    Ok(())
}
