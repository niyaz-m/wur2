use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;

pub async fn db_connection() {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("ERROR: DATABASE_URL not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("ERROR: failed to connect to Postgres");

    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("ERROR: failed to run test query");

    println!("INFO: database connected successfully, result = {}", row.0);
}
