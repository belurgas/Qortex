use std::fmt;

use mongodb::bson::{self, oid::ObjectId, Bson, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Role {
    DEFAULT,
    MODER,
    ADMIN,
}

// Имплементация Display для Role
impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let role_str = match self {
            Role::DEFAULT => "DEFAULT",
            Role::MODER => "MODER",
            Role::ADMIN => "ADMIN",
        };
        write!(f, "{}", role_str)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Bson,
    pub telegram_id: i64,
    pub username: Option<String>,
    pub role: Role,
    pub created_at: DateTime,
}

impl User {
    pub fn new(
        telegram_id: i64,
        username: Option<String>,
        role: Role,
    ) -> Self {
        Self {
            id: Bson::ObjectId(ObjectId::new()),
            telegram_id,
            username,
            role,
            created_at: DateTime::now(),
        }
    }
}