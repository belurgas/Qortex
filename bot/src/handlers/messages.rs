use std::sync::Arc;

use grpc_service::client::spawn_client_request_with_callback;
use logging::log_info;
use teloxide::{prelude::Requester, types::Message};
use tokio::sync::oneshot;

use crate::{types::{HandlerResult, MyDialogue}, TelegramBot};



pub async fn default_messages(bots: Arc<TelegramBot>, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let bot = &bots.bot;
    if let Some(text) = msg.text() {
        bot.send_message(msg.chat.id, "Думаю над ответом...").await?;
        
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