pub mod faq;

use std::{collections::HashMap, sync::Arc};

use logging::log_info;
use teloxide::{payloads::SendMessageSetters, prelude::Requester, types::{CallbackQuery, ParseMode}, utils::markdown::escape};
use async_trait::async_trait;
use crate::{handlers::callback::faq::{FaqSend, Q1}, keyboards::faqkb::faq, state::State, types::{HandlerResult, MyDialogue}, TelegramBot};

pub struct CallbackContext {
    pub bots: Arc<TelegramBot>,
    pub dialogue: MyDialogue,
    pub query: CallbackQuery,
}

#[async_trait]
pub trait CallbackHandler: Send + Sync {
    async fn handle(&self, ctx: &CallbackContext) -> HandlerResult;
}

pub struct CallbackHandlerFactory {
    handlers: HashMap<String, Arc<dyn CallbackHandler + Send + Sync>>,
}

impl CallbackHandlerFactory {
    pub fn new() -> Self {
        let mut handlers = HashMap::new();
        
        handlers.insert(
            "faq".to_string(),
            Arc::new(FaqSend) as Arc<dyn CallbackHandler + Send + Sync>
        );
        handlers.insert(
            "profits".to_string(),
            Arc::new(Q1) as Arc<dyn CallbackHandler + Send + Sync>
        );
        handlers.insert(
            "back_to_faq".to_string(),
            Arc::new(FaqSend) as Arc<dyn CallbackHandler + Send + Sync>
        );
        // handlers.insert("back_to_faq".to_string(), Arc::new(BackToFaqHandler));
        // Добавляем другие обработчики
        
        Self { handlers }
    }
    
    pub fn get_handler(&self, callback_data: &str) -> Option<&Arc<dyn CallbackHandler + Send + Sync>> {
        self.handlers.get(callback_data)
    }
}

pub async fn callback_handler(
    bots: Arc<TelegramBot>,
    dialogue: MyDialogue,
    q: CallbackQuery,
) -> HandlerResult {
    let bot = &bots.bot;
    bot.answer_callback_query(q.id.clone()).await?;

    let ctx = CallbackContext {
        bots: bots.clone(),
        dialogue,
        query: q,
    };

    if let Some(ref data) = ctx.query.data {
        match bots.callback_handlers.get_handler(&data) {
            Some(handler) => handler.handle(&ctx).await?,
            None => log_info!("Unknown callback from {} with data: {}", ctx.query.from.id.0, data),
        }
    }

    Ok(())
}