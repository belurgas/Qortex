use std::env;

use chrono::{DateTime, Utc};
use mongodb::{bson::{doc, Document}, Client, Collection};
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
    pub telegra_id: i64,
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