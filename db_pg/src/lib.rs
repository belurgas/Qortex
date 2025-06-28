use std::{error, fmt};
use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{FromRow, Row};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name= "user_role", rename_all="lowercase")]
pub enum UserRole {
    Default,
    Admin,
    WithAccess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "message_status", rename_all = "lowercase")]
pub enum MessageStatus {
    Pending,    // Ожидает рассмотрения
    Accepted,   // Принято в работу
    Answered,   // Ответ дан
}

impl fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Ожидает рассмотрения"),
            Self::Accepted => write!(f, "Принято в работу"),
            Self::Answered => write!(f, "Ответ дан"),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct User {
    pub telegram_id: i64,
    pub username: Option<String>,
    pub uuid: Uuid,
    pub role: UserRole,
}

#[derive(Debug, Clone, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub telegram_id: i64,
    pub text: String,
    pub status: MessageStatus,
    pub answer: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UserRepository {
    pub pool: PgPool,
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

        sqlx::query(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'message_status') THEN
                    CREATE TYPE message_status AS ENUM ('pending', 'accepted', 'answered');
                END IF;
            END
            $$;
            "#
        )
        .execute(&self.pool)
        .await
    .expect("Failed to create message_status enum type");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id UUID PRIMARY KEY,
                telegram_id BIGINT NOT NULL REFERENCES users(telegram_id) ON DELETE CASCADE,
                text TEXT NOT NULL,
                status message_status NOT NULL DEFAULT 'pending',
                answer TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .expect("Failed to create messages table");

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

    pub async fn add_message(&self, telegram_id: i64, text: &str) -> Result<Uuid> {
        let message_id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO messages (id, telegram_id, text)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(message_id)
        .bind(telegram_id)
        .bind(text)
        .execute(&self.pool)
        .await
        .expect("Failed to insert message");

        Ok(message_id)
    }

    pub async fn update_message_status(
        &self,
        message_id: Uuid,
        new_status: MessageStatus,
        answer: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE messages
            SET 
                status = $1,
                answer = $2,
                updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(new_status)
        .bind(answer)
        .bind(message_id)
        .execute(&self.pool)
        .await
        .expect("Failed to update message status");

        Ok(())
    }

    pub async fn get_user_messages(&self, telegram_id: i64) -> Result<Vec<Message>> {
        let messages = sqlx::query_as::<_, Message>(
            r#"
            SELECT id, telegram_id, text, status, answer, created_at, updated_at
            FROM messages
            WHERE telegram_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(telegram_id)
        .fetch_all(&self.pool)
        .await
        .expect("Failed to fetch messages");

        Ok(messages)
    }

    pub async fn get_message_by_id(&self, message_id: Uuid) -> Result<Option<Message>> {
        let message = sqlx::query_as::<_, Message>(
            r#"
            SELECT id, telegram_id, text, status, answer, created_at, updated_at
            FROM messages
            WHERE id = $1
            "#,
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await
        .expect("Failed to fetch message by ID");

        Ok(message)
    }

    pub async fn get_messages_by_status(&self, status: MessageStatus) -> Result<Vec<Message>> {
        let messages = sqlx::query_as::<_, Message>(
            r#"
            SELECT id, telegram_id, text, status, answer, created_at, updated_at
            FROM messages
            WHERE status = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .expect("Failed to fetch messages by status");

        Ok(messages)
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
