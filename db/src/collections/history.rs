use mongodb::bson::{oid::ObjectId, Bson, DateTime};
use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserHistory {
    #[serde(rename = "_id")]
    pub id: Bson,
    pub telegram_id: i64,
    pub messages: Vec<HistoryMessage>,
}


impl UserHistory {
    // Create new user history collection
    pub fn new(
        telegram_id: i64,
        system_prompt: String,
    ) -> Self {
        Self {
            id: Bson::ObjectId(ObjectId::new()),
            telegram_id,
            messages: vec![HistoryMessage {
                role: "system".to_string(),
                content: system_prompt,
                timestamp: DateTime::now(),
            }]
        }
    }
}