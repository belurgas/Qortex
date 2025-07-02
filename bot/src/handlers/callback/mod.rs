pub mod faq;
pub mod requests;

use std::{collections::HashMap, sync::Arc};

use logging::log_info;
use teloxide::{payloads::{SendMessageSetters, SendPhotoSetters}, prelude::Requester, types::{CallbackQuery, InputFile, ParseMode}, utils::markdown::escape};
use async_trait::async_trait;
use crate::{handlers::callback::{faq::{FaqSend, Q1}, requests::{AllMessages, BackToPageHandler, MessageHandler, MyRequests}}, keyboards::{faqkb::faq, menu::menu}, state::State, types::{HandlerResult, MyDialogue}, TelegramBot};

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

pub struct BackToMenu;

#[async_trait]
impl CallbackHandler for BackToMenu {
    async fn handle(&self, ctx: &CallbackContext) -> HandlerResult {
        ctx.bots.bot.delete_message(ctx.query.from.id, ctx.query.message.clone().unwrap().regular_message().unwrap().id).await?;

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/aw_logo.png");
        let image = InputFile::file(path);
        let text = format!(
            "*{}* привет\nМы команда разработчиков *Axiowel*, занимаемся разработкой эффективного и отказоустойчевого программного обеспечения основоного на ИИ модели *Axiowel AI*\n\nНаш бот достататочно функционален, можете подробнее узнать в /faq",
            escape(&ctx.query.from.first_name)
        );
        ctx.bots.bot.send_photo(ctx.query.from.id, image)
            .caption(text)
            .reply_markup(menu())
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        Ok(())
    }
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
        handlers.insert(
            "all_requests".to_string(),
            Arc::new(AllMessages) as Arc<dyn CallbackHandler + Send + Sync>
        );
        handlers.insert(
            "my_requests".to_string(),
            Arc::new(MyRequests) as Arc<dyn CallbackHandler + Send + Sync>
        );
        handlers.insert(
            "back_to_menu".to_string(),
            Arc::new(BackToMenu) as Arc<dyn CallbackHandler + Send + Sync>
        );
        handlers.insert(
            "msg_".to_string(), // Общая префикс-обработка для сообщений
            Arc::new(MessageHandler) as Arc<dyn CallbackHandler + Send + Sync>
        );

        handlers.insert(
            "back_to_page_".to_string(), // Общая префикс-обработка для кнопок возврата
            Arc::new(BackToPageHandler) as Arc<dyn CallbackHandler + Send + Sync>
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
        if data.starts_with("page_") {
            if let Some(handler) = bots.callback_handlers.get_handler("all_requests") {
                handler.handle(&ctx).await?;
                return Ok(());
            }
        }

        // Обработка отдельных сообщений
        if data.starts_with("msg_") {
            if let Some(handler) = bots.callback_handlers.get_handler("msg_") {
                handler.handle(&ctx).await?;
                return Ok(());
            }
        }

        // Обработка возврата
        if data.starts_with("back_to_page_") {
            if let Some(handler) = bots.callback_handlers.get_handler("back_to_page_") {
                handler.handle(&ctx).await?;
                return Ok(());
            }
        }

        match bots.callback_handlers.get_handler(&data) {
            Some(handler) => handler.handle(&ctx).await?,
            None => log_info!("Unknown callback from {} with data: {}", ctx.query.from.id.0, data),
        }
    }

    Ok(())
}