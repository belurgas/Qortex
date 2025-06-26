use db_pg::UserRepository;
use handlers::{commands::{command_handler, Commander}, messages};
use logging::{log_error, log_info, logger::setup_logger};
use dotenvy::dotenv;
use state::State;
use teloxide::{adaptors::{throttle::Limits}, dispatching::dialogue::InMemStorage, prelude::*};
use types::MyBot;
use std::{env, sync::Arc};

use crate::handlers::callback::{callback_handler, CallbackHandlerFactory};

pub mod keyboards;
mod handlers;
pub mod state;
pub mod types;

pub struct TelegramBot {
    pub bot: MyBot,
    pub storage: Arc<InMemStorage<State>>,
    pub db: Arc<UserRepository>,
    pub callback_handlers: Arc<CallbackHandlerFactory>,
}

impl TelegramBot {
    /// Create Bot Copy
    pub async fn new(bot_token: String, db: Arc<UserRepository>) -> Arc<Self> {
        let bot = Bot::new(bot_token).throttle(Limits::default());
        let storage = InMemStorage::<State>::new();
        let callback_handlers = Arc::new(CallbackHandlerFactory::new());
        Arc::new(TelegramBot { bot, storage, db, callback_handlers })
    }

    /// Bot Start
    pub async fn run(self: Arc<Self>) {
        let handler = dptree::entry()
            .branch(
                Update::filter_message()
                    .branch(
                    dptree::entry().filter_command::<Commander>().enter_dialogue::<Message, InMemStorage<State>, State>().endpoint(
                        |bot: Arc<TelegramBot>, dialogue, msg, cmd: Commander| async move {
                            command_handler(bot, dialogue, msg, cmd).await
                        }
                    ))
                    .branch(
                        dptree::entry().enter_dialogue::<Message, InMemStorage<State>, State>().endpoint(
                            |bot: Arc<TelegramBot>, dialogue, msg| async move {
                                messages::default_messages(bot, dialogue, msg).await
                            }
                        )
                    )
            ).branch(
                Update::filter_callback_query().enter_dialogue::<CallbackQuery, InMemStorage<State>, State>().endpoint(
                    |bot: Arc<TelegramBot>, dialogue, q| async move {
                        callback_handler(bot, dialogue, q).await
                    }
                )
            );

        // Dispatch builder and starter
        Dispatcher::builder(self.bot.clone(), handler)
            .dependencies(dptree::deps![
                self.clone(),
                self.storage.clone()
            ])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }
}

pub async fn start() {
    // Donenv, logger, load
    dotenv().ok();
    setup_logger().expect("Не удалось настроить логгер");

    let token = env::var("TOKEN").expect("Ошибка при получение токена из .env");
    let url = env::var("DB_URL").unwrap();


    log_info!("Бот запущен...");

    let repo = UserRepository::new(&url).await.unwrap();
    if let Err(e) = repo.init_table().await {
        log_error!("Ошибка иницализации таблицы: {}", e);
    }

    // Bot init
    let bot = TelegramBot::new(token, repo.into()).await;
    let _urn = bot.run().await;
}