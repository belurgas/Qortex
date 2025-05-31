use grpc_service::{hello_world::greeter_server::GreeterServer, MyGreeter};
use logging::{log_info, logger::setup_logger};
use monitor::{print_all, say_hello};
use dotenvy::dotenv;
use teloxide::{adaptors::{throttle::Limits, Throttle}, dispatching::dialogue::InMemStorage, dptree::case, prelude::*, utils::command::BotCommands};
use tokio::fs;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use std::{env, path::{Path, PathBuf}};
use rustls::crypto::CryptoProvider;

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

#[tokio::main]
async fn main() {
    // Donenv, logger, load
    dotenv().ok();
    setup_logger().expect("Не удалось настроить логгер");

    let token = env::var("TOKEN").expect("Ошибка при получение токена из .env");

    // Function thats start another thread for gRPC crate
    tokio::spawn(async move {
        rustls::crypto::ring::default_provider().install_default().unwrap();
        let addr = "127.0.0.1:5051".parse().unwrap();
        let greeter = MyGreeter::default();

        let (server_cert, server_key, ca_cert) = load_certs().await;

        let identity = Identity::from_pem(
            &server_cert,
            &server_key,
        );

        // Load root CA for clients checking
        let ca_cert = Certificate::from_pem(ca_cert);

        let tls = ServerTlsConfig::new()
            .identity(identity)
            .client_ca_root(ca_cert)
            .client_auth_optional(true); // Ultimate method for tls. If you use CA cert, this method need at

        log_info!("Запустили gRPC!");
        Server::builder()
            .tls_config(tls).unwrap()
            .add_service(GreeterServer::new(greeter))
            .serve(addr)
            .await.unwrap();
    });

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
        );
    
    // Dispatch builder and starter
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

// Получаем путь к папке certs в корне проекта
async fn get_certs_path() -> PathBuf {
    let mut path = std::env::current_dir().expect("Не могу получить текущий каталог");
    println!("{:?}", path.to_string_lossy());
    return path.parent().unwrap().join("tls");
}

async fn load_certs() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let certs_dir = get_certs_path().await;

    // Диагностические выводы
    println!("Директория certs: {}", certs_dir.display());

    if !certs_dir.exists() {
        panic!("Директория {} не существует!", certs_dir.display());
    }

    let server_crt = certs_dir.join("server/server.crt");
    let server_key = certs_dir.join("server/server.key");
    let ca_cert = certs_dir.join("ca/ca.crt");

    for path in &[&server_crt, &server_key, &ca_cert] {
        if path.exists() {
            println!("✅ Файл {} найден", path.display());
        } else {
            println!("❌ Файл {} НЕ найден", path.display());
        }
    }

    // Проверяем наличие всех файлов перед чтением
    if !server_crt.exists() {
        panic!("Файл {} не найден", server_crt.display());
    }
    if !server_key.exists() {
        panic!("Файл {} не найден", server_key.display());
    }
    if !ca_cert.exists() {
        panic!("Файл {} не найден", ca_cert.display());
    }

    // Теперь читаем
    let server_cert = fs::read(&server_crt).await.expect("Не могу прочитать server.crt");
    let server_key = fs::read(&server_key).await.expect("Не могу прочитать server.key");
    let ca_cert = fs::read(&ca_cert).await.expect("Не могу прочитать ca.cert");

    println!("Все файлы успешно загружены!");

    (server_cert, server_key, ca_cert)
}