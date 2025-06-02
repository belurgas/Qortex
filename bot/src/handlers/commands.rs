use std::sync::Arc;

use logging::log_info;
use teloxide::{prelude::Requester, types::Message, utils::command::BotCommands};

use crate::{types::{HandlerResult, MyDialogue}, TelegramBot};

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
}

pub async fn command_handler(bots: Arc<TelegramBot>, dialogue: MyDialogue, msg: Message, cmd: Commander) -> HandlerResult {
    let bot = &bots.bot;
    match cmd {
        Commander::Help => bot.send_message(msg.chat.id, Commander::descriptions().to_string()).await?,
        Commander::Start => {
            log_info!("Состояние: {:?}", dialogue.get().await.unwrap());
            bot.send_message(msg.chat.id, format!("Запустили")).await?;

            return Ok(());
        }
        Commander::SendMessage(message) => {
            bot.send_message(msg.chat.id, format!("Сообщение: {} отправлено", message))
                .await?;

            return Ok(());
        }
    };

    Ok(())
}