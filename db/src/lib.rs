use std::{env, sync::{Arc, Mutex}};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use collections::{answer::AnswerRequest, history::UserHistory, user::User};
use mongodb::{bson::{doc, oid::ObjectId, to_bson, Document}, options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};

pub mod collections;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum StatusCode {
    Exist,
    ObjectIdError,
    UserId(ObjectId),
    GroupId(ObjectId),
}

pub struct Database {
    client: Client,
    users_collection: Arc<Collection<User>>,
    history_collection: Arc<Collection<UserHistory>>,
    answers_collection: Arc<Collection<AnswerRequest>>,
}

impl Database {
    // Database init
    pub async fn new(
        uri: &str,
        db_name: &str
    ) -> Result<Arc<Self>> {
        let client_options = ClientOptions::parse(uri).await?;
        let client = Client::with_options(client_options)?;
        let database = client.database(db_name);

        // Collections check
        let collections = database.list_collection_names().await?;
        if !collections.contains(&"users".to_string()) {
            database.create_collection("users").await?;
        }

        if !collections.contains(&"user_history".to_string()) {
            database.create_collection("user_history").await?;
        }

        if !collections.contains(&"answers".to_string()) {
            database.create_collection("answers").await?;
        }

        // Get collections
        let users_collections = database.collection::<User>("users");
        let user_history = database.collection::<UserHistory>("user_history");
        let answers_collection = database.collection::<AnswerRequest>("answers");

        Ok(
            Arc::new( Self {
                client,
                users_collection: Arc::new(users_collections),
                history_collection: Arc::new(user_history),
                answers_collection: Arc::new(answers_collection),
            })
        )
    }

    // Add user if he doesn't exist in DB
    pub async fn add_user(
        &self,
        user: User
    ) -> Result<StatusCode> {
        let existing_user = self.users_collection.find_one(doc! { "telegram_id": user.telegram_id }).await?;
        if existing_user.is_some() {
            return Ok(StatusCode::Exist);
        }

        let result = self.users_collection.insert_one(user).await?;
        if let Some(id) = result.inserted_id.as_object_id() {
            println!("User created successfully with id: {}", id);
            return Ok(StatusCode::UserId(id));
        } else {
            return Ok(StatusCode::ObjectIdError);
        }
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