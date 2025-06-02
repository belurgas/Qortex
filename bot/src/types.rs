use teloxide::{adaptors::Throttle, dispatching::dialogue::InMemStorage, prelude::Dialogue, Bot};

use crate::state::State;

/// MyDialogue type need for using FSM Context managment
pub type MyDialogue = Dialogue<State, InMemStorage<State>>;

/// Simple Result<> type for functions
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Using Trottle describer for rate limit setting
pub type MyBot = Throttle<Bot>;