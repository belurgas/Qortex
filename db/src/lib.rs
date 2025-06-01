use std::{env, sync::{Arc, Mutex}};

use anyhow::Context;
use chrono::{DateTime, Utc};
use mongodb::{bson::{doc, Document, to_bson}, options::ClientOptions, Client, Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub telegram_id: i64,
    pub username: Option<String>,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum UserRole {
    DEFAULT,
    MODER,
    ADMIN,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserHistory {
    pub telegram_id: i64,
    pub messages: Vec<HistoryMessage>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnswerRequest {
    pub telegram_id: i64,
    pub text: String,
    pub status: AnswerStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AnswerStatus {
    ACCEPTED,
    REVIEWED,
}

struct DatabasePool {
    client: Client,
    db_name: String,
}

impl DatabasePool {
    pub async fn new(uri: &str, db_name: &str) -> anyhow::Result<Arc<Mutex<Self>>> {
        let mut client_options = ClientOptions::parse(uri).await?;
        client_options.app_name = Some("ai-bot".to_string());

        let client = Client::with_options(client_options)?;

        // Check connection
        client.list_database_names().await
            .expect("Не удалось подключиться к MongoDB");

        Ok(Arc::new(Mutex::new(DatabasePool { client, db_name: db_name.to_string() })))
    }

    pub fn db(&self) -> Database {
        self.client.database(&self.db_name)
    }

    pub fn users(&self) -> Collection<User> {
        self.db().collection("users")
    }

    pub fn history(&self) -> Collection<UserHistory> {
        self.db().collection("user_history")
    }

    pub fn answers(&self) -> Collection<AnswerRequest> {
        self.db().collection("answers")
    }
}

pub struct DatabaseService {
    pool: Arc<Mutex<DatabasePool>>,
}

impl DatabaseService {
    pub async fn new(uri: &str, db_name: &str) -> anyhow::Result<Self> {
        let pool = DatabasePool::new(uri, db_name).await?;
        Ok(Self {
            pool
        })
    }

    //1. Checking/Registration user
    pub async fn check_or_register_user(&self, user_id: i64, username: Option<String>) -> anyhow::Result<()> {
        let pool = self.pool.lock().unwrap();
        let users = pool.users();

        let created_at_bson = to_bson(&Utc::now()).unwrap();
        let role = to_bson(&UserRole::DEFAULT).unwrap();

        let filter = doc! { "telegram_id": user_id };
        let update = doc! {
            "$setOnInsert": {
                "telegram_id": user_id,
                "username": username,
                "role": role,
                "created_at": created_at_bson,
            }
        };

        users.update_one(filter, update).upsert(true)
            .await
            .context("Ошибка при регистрации пользователя")?;

        Ok(())
    }

    //2. Change user role
    pub async fn update_user_role(&self, username: &str, new_role: UserRole) -> anyhow::Result<bool> {
        let pool = self.pool.lock().unwrap();
        let users = pool.users();
        
        let new_role = to_bson(&new_role)?;

        let filter = doc! { "username": username };
        let update = doc! { "$set": { "role": new_role } };
        
        let result = users.update_one(filter, update).await?;
        Ok(result.modified_count > 0)
    }

    // 4. Clear history
    pub async fn clear_user_history(&self, user_id: i64) -> anyhow::Result<bool> {
        let pool = self.pool.lock().unwrap();
        let history = pool.history();
        
        let filter = doc! { "telegram_id": user_id };
        let result = history.delete_one(filter).await?;
        Ok(result.deleted_count > 0)
    }

    //5. Add user history
    pub async fn add_message_to_history(
        &self,
        user_id: i64,
        role: &str,
        content: &str,
        is_system: bool
    ) -> anyhow::Result<()> {
        let pool = self.pool.lock().unwrap();
        let history = pool.history();
        
        let message = HistoryMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        };

        let filter = doc! { "telegram_id": user_id };

        if is_system {
            // Create new history with system prompt
            let new_history = UserHistory {
                telegram_id: user_id,
                messages: vec![message],
            };

            history.replace_one(filter, new_history).upsert(true)
                .await
                .context("Ошибка при добавлении системного промпта")?;
        } else {
            // Add mes in act hist
            let update = doc! {
                "$push": {
                    "messages": to_bson(&message)?
                }
            };

            history.update_one(filter, update)
                .await
                .context("Ошибка при добавлении сообщения в историю")?;
        }

        Ok(())
    }

    // Получение истории для ИИ
    pub async fn get_ai_history(&self, user_id: i64) -> anyhow::Result<Vec<HistoryMessage>> {
        let pool = self.pool.lock().unwrap();
        let history = pool.history();
        
        let filter = doc! { "telegram_id": user_id };
        let result = history.find_one(filter).await?;
        
        Ok(result.map(|h| h.messages).unwrap_or_default())
    }
}

pub async fn db_test() -> mongodb::error::Result<()> {
    let uri = env::var("MONGODB_URI").expect("Incorrect database_string");
    println!("uri: {}", uri);
    // Create a new client and connect to the server
    let client = Client::with_uri_str(uri).await?;
    
    // Get a handle on the movies collection
    let database = client.database("sample_mflix");
    let my_coll: Collection<Document> = database.collection("movies");
    // Find a movie based on the title value
    let my_movie = my_coll.find_one(doc! { "title": "The Perils of Pauline" }).await?;
    // Print the document
    println!("Found a movie:\n{:#?}", my_movie);
    Ok(())
}

// 3 collections

// UserHistory
// Users
// Answers

// Users add, role change, data change
// Answers add, change status, clear (with archiving)
// UserHistory, add, clear, limit clearing