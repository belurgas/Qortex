use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("None option while return message from data")]
    UnvialabelFromData,
    #[error("unknown data store error")]
    Unknown,
}