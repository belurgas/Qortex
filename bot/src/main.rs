use grpc_service::{start_grpc, MyPromptService};
use logging::{log_info, logger::setup_logger};
use monitor::{print_all, say_hello};
use dotenvy::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use teloxide::{adaptors::{throttle::Limits, Throttle}, dispatching::dialogue::InMemStorage, dptree::case, prelude::*, types::ParseMode, utils::{command::BotCommands, markdown::escape}};
use tokio::fs;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use std::{env, path::{Path, PathBuf}, sync::Arc};
use rustls::crypto::CryptoProvider;

mod mistral;


#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    OnWaiting,
    Send,
}

/// Commands for bot
#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case", description = "These commands are supported:")]
enum Command {
    #[command(description = "Отображает этот текс")]
    Help,
    #[command(description = "Запускает бота")]
    Start,
    #[command(description = "Отправляет сообщение админу",)]
    SendMessage(String),
}

/// MyDialogue type need for using FSM Context managment
type MyDialogue = Dialogue<State, InMemStorage<State>>;

/// Simple Result<> type for functions
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Using Trottle describer for rate limit setting
type MyBot = Throttle<Bot>;

/// Answering for commands requests
async fn answer(bot: MyBot, msg: Message, cmd: Command, fsm: MyDialogue) -> HandlerResult {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
        Command::Start => {
            log_info!("Состояние: {:?}", fsm.get().await.unwrap());
            bot.send_message(msg.chat.id, format!("Запустили")).await?
        }
        Command::SendMessage(message) => {
            bot.send_message(msg.chat.id, format!("Сообщение: {} отправлено", message))
                .await?
        }
    };

    Ok(())
}

async fn send(bot: MyBot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start! Send piska?").await?;
    // dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

async fn handle_ai_message(bot: MyBot, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        bot.send_message(msg.chat.id, "Думаю над ответом...").await?;

        match mistral::query_mistral_api(text).await {
            Ok(response) => {
                // Попытка отправить как MarkdownV2
                bot
                    .send_message(msg.chat.id, escape(&response))
                    .parse_mode(ParseMode::MarkdownV2)
                    .await.unwrap();

                // if let Err(e) = send_result {
                //     if e.to_string().contains("can't parse entities") {
                //         // Если ошибка разбора Markdown — экранируем и пробуем снова
                //         let safe_text = escape(&response);
                //         bot.send_message(msg.chat.id, safe_text)
                //             .parse_mode(ParseMode::MarkdownV2)
                //             .await?;
                //     } else {
                //         return Err(e.into());
                //     }
                // }
            }
            Err(e) => {
                bot.send_message(msg.chat.id, format!("Ошибка: {}", e)).await?;
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Donenv, logger, load
    dotenv().ok();
    setup_logger().expect("Не удалось настроить логгер");

    let token = env::var("TOKEN").expect("Ошибка при получение токена из .env");
    
    // временно отключено... ⚠️⚠️⚠️
    // tokio::spawn(async move {
    //     let _ = start_grpc().await;
    // });

    log_info!("Бот запущен...");

    // Bot init
    let bot = Bot::new(token).throttle(Limits::default());

    // Dptree handler
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .branch(dptree::case![State::Send].endpoint(send))
        )
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .endpoint(answer)
        )
        .branch(
            Update::filter_message()
                .filter(|msg: Message| msg.text().is_some())
                .endpoint(handle_ai_message),
        );
    
    // Dispatch builder and starter
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}