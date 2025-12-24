mod auth;
mod db;
mod messages;
mod models;
mod server;
mod users;

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

use crate::db::UserDb;
use crate::server::start_server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    //db_connection().await;
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("ERROR: DATABASE_URL not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("ERROR: failed to connect to Postgres");

    // test query
    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("ERROR: failed to run test query");

    let _ = sqlx::migrate!("./migrations").run(&pool).await;

    println!("INFO: database connected successfully, result = {}", row.0);

    UserDb::new(pool.clone());

    println!("INFO: server starting on port :6969");

    if let Err(e) = start_server("127.0.0.1:6969", pool).await {
        eprintln!("ERROR: failed to start server: {}", e);
    }
    Ok(())
}
