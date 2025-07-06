use std::{sync::Arc, time::Duration};
use db_pg::User;
use logging::{log_error, log_info};
use teloxide::{payloads::{SendMessageSetters, SendPhotoSetters}, prelude::Requester, types::{InputFile, Message, ParseMode}, utils::{command::BotCommands, markdown::escape}};
use tokio::time::sleep;
use uuid::Uuid;

use crate::{keyboards::{faqkb::faq, menu::menu}, state::State, types::{HandlerResult, MyDialogue}, TelegramBot};

/// Commands for bot
#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case", description = "These commands are supported:")]
pub enum Commander {
    #[command(description = "/faq - вам поможет")]
    Help,
    #[command(description = "Запуск бота")]
    Start,
    #[command(description = "Оставить сообщение администратору",)]
    Send(String),
    #[command(description = "FAQ ℹ️ бота",)]
    Faq,
}

pub async fn command_handler(bots: Arc<TelegramBot>, dialogue: MyDialogue, msg: Message, cmd: Commander) -> HandlerResult {
    let bot = &bots.bot;
    if msg.chat.id.0 as u64 != msg.from.clone().unwrap().id.0 {
        log_info!("ChatId not eq UserId");
        return Ok(());
    }
    match cmd {
        Commander::Help => bot.send_message(msg.chat.id, Commander::descriptions().to_string()).await?,
        Commander::Start => {
            let new_user = User {
                telegram_id: msg.chat.id.0,
                username: msg.from.unwrap().username,
                uuid: Uuid::new_v4(),
                role: db_pg::UserRole::Default,
            };

            let pool = bots.db.pool.clone();
            tokio::spawn(async move {
                let user = new_user;
                sleep(Duration::from_secs(5)).await;
                let res: Result<_, sqlx::Error> = 
                sqlx::query(
                    r#"
                    INSERT INTO users (telegram_id, username, uuid, role)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (telegram_id) DO NOTHING
                    "#
                )
                .bind(user.telegram_id)
                .bind(&user.username.clone().unwrap_or("None".to_string()))
                .bind(user.uuid)
                .bind(user.role)
                .execute(&pool)
                .await;
                if let Err(e) = res {
                    log_error!("Error while insert user into db: {:?}", e);
                } else {
                    log_info!("It's ok!");
                }
                
            });

            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/static/aw_logo.png");
            let image = InputFile::file(path);
            let text = format!(
                "*{}* привет\nМы команда разработчиков *Axiowel*, занимаемся разработкой эффективного и отказоустойчевого программного обеспечения основоного на ИИ модели *Axiowel AI*\n\nНаш бот достататочно функционален, можете подробнее узнать в /faq",
                escape(&msg.chat.first_name().unwrap_or(""))
            );
            bot.send_message(msg.chat.id, text)
                .parse_mode(ParseMode::MarkdownV2)
                .reply_markup(menu())
                .await;

            return Ok(());
        }
        Commander::Send(message) => {
            let message_uuid = bots.db.add_message(msg.chat.id.0, &message).await?;
            bot.send_message(msg.chat.id, format!("Сообщение с уникальным номером: `{}` отправлено, ожидвйте ответа\\!", escape(message_uuid.to_string().as_str())))
                .parse_mode(ParseMode::MarkdownV2)
                .await?;

            return Ok(());
        },
        Commander::Faq => {
            let mut first_name = escape(&msg.chat.first_name().unwrap_or(""));
            if first_name != "" {
                first_name.push(' ');
            }
            let text = format!(
                "*FAQ ℹ️*\n\n*{}*Если вы не нашли ответ на свой вопрос, то можете написать прямо сюда в чат 💭 и *Axiobot* 🤖 поможет ответить на ваш вопрос 😊",
                first_name
            );
            
            bot.send_message(msg.chat.id, text)
                .reply_markup(faq())
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
                
            dialogue.update(State::WaitQuestion).await?;

            return Ok(())
        }
    };

    Ok(())
}