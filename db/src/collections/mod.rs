use mongodb::bson::Bson;
use user::Role;

pub mod user;
pub mod answer;
pub mod history;

// Convert Role to Bson string
impl From<Role> for Bson {
    fn from(role: Role) -> Self {
        Bson::String(role.to_string())
    }
}