use std::error;
use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{FromRow, Row};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name= "user_role", rename_all="lowercase")]
pub enum UserRole {
    Default,
    Admin,
    WithAccess,
}

#[derive(Debug, FromRow)]
pub struct User {
    pub telegram_id: i64,
    pub username: Option<String>,
    pub uuid: Uuid,
    pub role: UserRole,
}

#[derive(Debug, Clone)]
pub struct UserRepository {
    pool: PgPool,
}

type Result<T> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

impl UserRepository {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(url)
            .await
            .expect("Failed to create db pool");
        Ok(Self { pool })
    }

    pub async fn init_table(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                telegram_id BIGINT PRIMARY KEY,
                username TEXT NOT NULL,
                uuid UUID NOT NULL UNIQUE,
                role TEXT NOT NULL DEFAULT 'default'
            );
            "#
        )
        .execute(&self.pool)
        .await
        .expect("Failed to create users table");

        Ok(())
    }

    pub async fn add_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (telegram_id, username, uuid, role)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (telegram_id) DO NOTHING
            "#
        )
        .bind(user.telegram_id)
        .bind(&user.username.clone().unwrap_or("None".to_string()))
        .bind(user.uuid)
        .bind(user.role)
        .execute(&self.pool)
        .await
        .expect("Failed to insert user");
        
        Ok(())
    }

    pub async fn delete_user(&self, telegram_id: i64) -> Result<()> {
        sqlx::query(
            "DELETE FROM users WHERE telegram_id = $1"
        )
        .bind(telegram_id)
        .execute(&self.pool)
        .await
        .expect("Failed to delete user");
        
        Ok(())
    }

    pub async fn check_role(&self, telegram_id: i64, required_role: UserRole) -> Result<bool> {
        let role: UserRole = sqlx::query_scalar(
            "SELECT role FROM users WHERE telegram_id = $1"
        )
        .bind(telegram_id)
        .fetch_optional(&self.pool)
        .await
        .expect("Failed to fetch user role")
        .unwrap_or(UserRole::Default);
        
        Ok(role == required_role)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT telegram_id, username, uuid, role FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .expect("Failed to find user by username");
        
        Ok(user)
    }

    pub async fn get_user(&self, user_uuid: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT telegram_id, username, uuid, role FROM users WHERE uuid = $1"
        )
        .bind(user_uuid)
        .fetch_optional(&self.pool)
        .await
        .expect("Failed to get user");
        
        Ok(user)
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
