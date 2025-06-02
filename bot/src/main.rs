use db::Database;
use handlers::{commands::{command_handler, Commander}, messages};
use logging::{log_info, logger::setup_logger};
use dotenvy::dotenv;
use state::State;
use teloxide::{adaptors::{throttle::Limits}, dispatching::dialogue::InMemStorage, prelude::*};
use types::MyBot;
use std::{env, sync::Arc};

mod keyboards;
mod handlers;
mod state;
mod types;

pub struct TelegramBot {
    pub bot: MyBot,
    pub storage: Arc<InMemStorage<State>>,
    pub db: Arc<Database>,
}

impl TelegramBot {
    /// Create Bot Copy
    pub async fn new(bot_token: String, db: Arc<Database>) -> Arc<Self> {
        let bot = Bot::new(bot_token).throttle(Limits::default());
        let storage = InMemStorage::<State>::new();
        Arc::new(TelegramBot { bot, storage, db })
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

#[tokio::main]
async fn main() {
    // Donenv, logger, load
    dotenv().ok();
    setup_logger().expect("Не удалось настроить логгер");

    let token = env::var("TOKEN").expect("Ошибка при получение токена из .env");
    let uri = env::var("MONGODB_URI").unwrap();


    log_info!("Бот запущен...");

    let db = db::Database::new(&uri, "ai_base").await.unwrap();

    // Bot init
    let bot = TelegramBot::new(token, db).await;
    let _urn = bot.run().await;
}