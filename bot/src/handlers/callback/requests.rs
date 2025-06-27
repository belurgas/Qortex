use async_trait::async_trait;
use teloxide::{payloads::SendMessageSetters, prelude::Requester, types::{CallbackQuery, ParseMode}, utils::markdown::escape};

use crate::{handlers::callback::{CallbackContext, CallbackHandler}, keyboards::{faqkb::{faq, profits}, requests::{all_messages, history}}, state::State, types::{HandlerResult, MyDialogue}, TelegramBot};

pub struct MyRequests;
pub struct AllMessages;

#[async_trait]
impl CallbackHandler for MyRequests {
    async fn handle(&self, ctx: &CallbackContext) -> HandlerResult {
        ctx.bots.bot.delete_message(ctx.query.from.id, ctx.query.regular_message().unwrap().id).await?;

        ctx.bots.bot.send_message(ctx.query.from.id, "*Выберете:*")
            .parse_mode(ParseMode::MarkdownV2)
            .reply_markup(history())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl CallbackHandler for AllMessages {
    async fn handle(&self, ctx: &CallbackContext) -> HandlerResult {
        ctx.bots.bot.delete_message(ctx.query.from.id, ctx.query.regular_message().unwrap().id).await?;
        let messages_from_db = ctx.bots.db.get_user_messages(ctx.query.from.id.0.try_into().unwrap()).await?;

        ctx.bots.bot.send_message(ctx.query.from.id, "*Выберете:*")
            .parse_mode(ParseMode::MarkdownV2)
            .reply_markup(all_messages(messages_from_db, 0))
            .await?;
        Ok(())
    }
}