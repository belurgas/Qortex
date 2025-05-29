use logging::{log_info, logger::setup_logger};
use monitor::{print_all, say_hello};
use dotenvy::dotenv;
use teloxide::{adaptors::{throttle::Limits, Throttle}, dispatching::dialogue::InMemStorage, dptree::case, prelude::*, utils::command::BotCommands};
use std::env;

#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    OnWaiting,
    Send,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Запускает бота")]
    Start,
    #[command(description = "Отправляет сообщение админу",)]
    SendMessage(String),
}


type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
type MyBot = Throttle<Bot>;

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

#[tokio::main]
async fn main() {
    dotenv().ok();
    setup_logger().expect("Не удалось настроить логгер");
    let token = env::var("TOKEN").expect("Ошибка при получение токена из .env");

    log_info!("Бот запущен...");

    let bot = Bot::new(token).throttle(Limits::default());

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
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
