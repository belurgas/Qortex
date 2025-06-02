use mongodb::bson::{oid::ObjectId, Bson, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnswerRequest {
    #[serde(rename = "_id")]
    pub id: Bson,
    pub telegram_id: i64,
    pub text: String,
    pub status: AnswerStatus,
    pub timestamp: DateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AnswerStatus {
    ACCEPTED,
    REVIEWED,
}

impl AnswerRequest {
    pub fn new(
        telegram_id: i64,
        text: String,
    ) -> Self {
        Self {
            id: Bson::ObjectId(ObjectId::new()),
            telegram_id,
            text,
            status: AnswerStatus::ACCEPTED,
            timestamp: DateTime::now(),
        }
    }
}