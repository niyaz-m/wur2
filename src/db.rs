use sqlx::PgPool;
use crate::models::User;

#[derive(Debug, Clone)]
pub struct UserDb {
    pool: PgPool,
}

impl UserDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        username: String,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, password_hash)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(&username)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        println!("INFO: Created user: {}", user.username);
        Ok(user)
    }

    pub async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, created_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, sqlx::Error> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, created_at
            FROM users
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}
