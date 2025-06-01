use db::DatabaseService;
use grpc_service::{client::spawn_client_request_with_callback};
use logging::{log_info, logger::setup_logger};
// use monitor::{print_all, say_hello};
use dotenvy::dotenv;
use teloxide::{adaptors::{throttle::Limits, Throttle}, dispatching::dialogue::InMemStorage, prelude::*, utils::{command::BotCommands}};
use tokio::{sync::oneshot, time::Instant};
// use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use std::{env, sync::Arc};


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

async fn send(bot: MyBot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Let's start! Send piska?").await?;
    // dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

async fn handle_ai_message(bot: MyBot, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        bot.send_message(msg.chat.id, "Думаю над ответом...").await?;
        
        // let telegram_id = msg.chat.id.0;
        // let username = msg.from.clone().unwrap().username;

//         let system_prompt = r#"
// Ты — ассистент, который отвечает строго в plain-тексте. Соблюдай правила:
// 1. **Запрещено любое форматирование**:
//    - Никаких Markdown, HTML, LaTeX.
//    - Никаких ```code blocks```, `inline_code`, > цитат.
//    - Никаких *курсива*, **жирного**, ~зачёркивания~.
//    - Никаких таблиц, списков с пунктами (1., - [x] и т.д.).
// 2. **Разрешено только**:
//    - Пустые строки для разделения логических блоков.
//    - Эмодзи (например, ✅, 🔥, ❗) для акцента.
// 3. **Если просят оформить текст**:
//    - Вежливо откажи: "Извините, я работаю только с plain-текстом".
// 4. Если пользователь просит написать, сгенерировать, обхяснить как что-то написать на каком-то языке, то вежливо откажи.
// 5. **Пример корректного ответа**:
//    "Сегодня солнечно ☀️  
   
//    Рекомендую прогулку в парк.  
//    Не забудьте воду 💧
            
//         "#;

//         db_service.check_or_register_user(telegram_id, username).await?;

//         db_service.add_message_to_history(telegram_id, "system", system_prompt, true).await?;
//         db_service.add_message_to_history(telegram_id, "user", msg.text().unwrap(), false).await?;
        let (tx, rx) = oneshot::channel();
        log_info!("Старт запроса");
        spawn_client_request_with_callback(tx, text.to_string());

        let bot_clone = bot.clone();
        tokio::spawn(async move {
            match rx.await {
                Ok(Ok(text)) => bot_clone.send_message(msg.chat.id, text).await.unwrap(),
                Ok(Err(e)) => bot_clone.send_message(msg.chat.id, e).await.unwrap(),
                Err(_) => bot_clone.send_message(msg.chat.id, "Ошибка связи с сервером").await.unwrap(),
            }
        });
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

    let uri = env::var("MONGODB_URI").unwrap();

    // let db_service = Arc::new(DatabaseService::new(&uri, "ai_bot").await.unwrap());


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
                .endpoint(handle_ai_message)
        );
    
    // Dispatch builder and starter
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            InMemStorage::<State>::new()
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}