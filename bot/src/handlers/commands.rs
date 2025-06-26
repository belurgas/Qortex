use std::sync::Arc;
use db_pg::User;
use logging::{log_error, log_info};
use teloxide::{payloads::SendMessageSetters, prelude::Requester, types::Message, utils::command::BotCommands};
use uuid::Uuid;

use crate::{keyboards::faqkb::test, types::{HandlerResult, MyDialogue}, TelegramBot};

/// Commands for bot
#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case", description = "These commands are supported:")]
pub enum Commander {
    #[command(description = "Отображает этот текс")]
    Help,
    #[command(description = "Запускает бота")]
    Start,
    #[command(description = "Отправляет сообщение админу",)]
    SendMessage(String),
    Faq,
}

pub async fn command_handler(bots: Arc<TelegramBot>, dialogue: MyDialogue, msg: Message, cmd: Commander) -> HandlerResult {
    let bot = &bots.bot;
    match cmd {
        Commander::Help => bot.send_message(msg.chat.id, Commander::descriptions().to_string()).await?,
        Commander::Start => {
            let new_user = User {
                telegram_id: msg.chat.id.0,
                username: msg.from.unwrap().username,
                uuid: Uuid::new_v4(),
                role: db_pg::UserRole::Default,
            };
            if let Err(e) = bots.db.add_user(&new_user).await {
                log_error!("Ошибка добавления пользователя в бд: {}", e);
            } else {
                log_info!("Пользователь {} в бд", msg.chat.id.0);
            }
            bot.send_message(msg.chat.id, format!("Запустили")).await?;

            return Ok(());
        }
        Commander::SendMessage(message) => {
            bot.send_message(msg.chat.id, format!("Сообщение: {} отправлено", message))
                .await?;

            return Ok(());
        },
        Commander::Faq => {
            bot.send_message(msg.chat.id, "Фак ю битч")
                .reply_markup(test())
                .await?;
            return Ok(())
        }
    };

    Ok(())
}