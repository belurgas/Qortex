use async_trait::async_trait;
use teloxide::{dispatching::dialogue, payloads::{EditMessageReplyMarkupSetters, SendMessageSetters}, prelude::Requester, types::{CallbackQuery, ParseMode}, utils::markdown::escape};
use uuid::Uuid;

use crate::{handlers::callback::{CallbackContext, CallbackHandler}, keyboards::{faqkb::{faq, profits}, requests::{all_messages, create_navigation_row, history, ITEMS_PER_PAGE}}, state::State, types::{HandlerResult, MyDialogue}, TelegramBot};

pub struct MyRequests;
pub struct AllMessages;
pub struct MessageSelectionHandler;
pub struct PageChangeHandler;

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
        let data = ctx.query.data.as_ref().unwrap();
        let mut state = ctx.dialogue.get().await?.unwrap_or_default();

        // Обработка действий
        match data.as_ref() {
            // Начальная загрузка сообщений
            "all_requests" => {
                let user_id: i64 = ctx.query.from.id.0.try_into().unwrap();
                let messages = ctx.bots.db.get_user_messages(user_id).await?;
                
                state = State::ViewingMessages {
                    messages,
                    current_page: 0,
                };
                ctx.dialogue.update(state.clone()).await?;
            },
            
            // Обработка пагинации
            d if d.starts_with("page_") => {
                if let State::ViewingMessages { messages, current_page } = &state {
                    let new_page = d["page_".len()..].parse::<usize>().unwrap_or(*current_page);
                    let total_pages = (messages.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
                    let valid_page = new_page.min(total_pages.saturating_sub(1));

                    state = State::ViewingMessages { 
                        messages: messages.clone(), 
                        current_page: valid_page 
                    };
                    ctx.dialogue.update(state.clone()).await?;
                } else {
                    eprintln!("Received page callback with invalid state");
                    return Ok(());
                }
            },
            
            // Неизвестный callback
            _ => return Ok(()),
        }

        // Обновляем клавиатуру
        if let State::ViewingMessages { messages, current_page } = &state {
            let keyboard = all_messages(messages.clone(), *current_page);

            if let Some(message) = ctx.query.message.as_ref() {
                let msg = message.regular_message().unwrap();
                ctx.bots.bot.edit_message_reply_markup(msg.chat.id, message.id())
                    .reply_markup(keyboard)
                    .await?;
            }
        }

        Ok(())
    }
}
