use async_trait::async_trait;
use teloxide::{payloads::SendMessageSetters, prelude::Requester, types::{CallbackQuery, ParseMode}, utils::markdown::escape};

use crate::{handlers::callback::{CallbackContext, CallbackHandler}, keyboards::faqkb::{faq, profits}, state::State, types::{HandlerResult, MyDialogue}, TelegramBot};

pub struct FaqSend;
pub struct Q1;

#[async_trait]
impl CallbackHandler for FaqSend {
    async fn handle(&self, ctx: &CallbackContext) -> HandlerResult {
        ctx.bots.bot.delete_message(ctx.query.from.id, ctx.query.regular_message().unwrap().id).await?;
        
        let mut first_name = escape(&ctx.query.from.first_name);
        if first_name != "" {
            first_name.push(' ');
        }
        let text = format!(
            "*FAQ ℹ️*\n\n*{}*Если вы не нашли ответ на свой вопрос 🫥, то можете написать прямо сюда в чат 💭 и наш помощник 🤖 поможет ответить на ваш вопрос 😊",
            first_name
        );
        
        ctx.bots.bot.send_message(ctx.query.from.id, text)
            .reply_markup(faq())
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
            
        ctx.dialogue.update(State::WaitQuestion).await?;
        
        Ok(())
    }
}

#[async_trait]
impl CallbackHandler for Q1 {
    async fn handle(&self, ctx: &CallbackContext) -> HandlerResult {
        ctx.bots.bot.delete_message(ctx.query.from.id, ctx.query.regular_message().unwrap().id).await?;
        let text = format!(
            "*🤔 Чем полезен этот бот?*\n{}",
            escape("Да всем...")
        );
        ctx.bots.bot.send_message(ctx.query.from.id, text)
            .reply_markup(profits())
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        Ok(())
    }
}
