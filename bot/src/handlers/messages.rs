use std::{sync::Arc, time::Duration};

use grpc_service::client::spawn_client_request_with_callback;
use logging::log_info;
use teloxide::{payloads::{EditMessageTextSetters, SendMessageSetters}, prelude::Requester, types::{Message, ParseMode}};
use tokio::{sync::oneshot, time};

use crate::{keyboards::faqkb::feedback_ai, state::State, types::{HandlerResult, MyDialogue}, TelegramBot};



pub async fn default_messages(bots: Arc<TelegramBot>, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let bot = &bots.bot;

    if let Some(state) = dialogue.get().await? {
        match state {
            State::WaitQuestion => {
                if let Some(question) = msg.text() {
                    log_info!("Пользователь {} обратился за помощью к Qortex AI с вопросом: {}", msg.chat.first_name().unwrap_or(msg.chat.id.0.to_string().as_str()), question);
                    let message = bot.send_message(msg.chat.id, "*Qortex AI*\n_Думаю над ответом\\.\\.\\._")
                        .parse_mode(ParseMode::MarkdownV2)
                        .await?;

                    // Обработка через ИИ
                    time::sleep(Duration::from_secs(1)).await;
                    log_info!("Ответ от AI получен");

                    bot.edit_message_text(msg.chat.id, message.id, "*Ваш ответ на вопрос:*\nОтвет на вопрос\n\n_Вы удволетворены ответом?_")
                        .parse_mode(ParseMode::MarkdownV2)
                        .reply_markup(feedback_ai())
                        .await?;
                }
            }
            _ => {}
        }
    }

    // if let Some(text) = msg.text() {
    //     bot.send_message(msg.chat.id, "Думаю над ответом...").await?;
        
    //     let (tx, rx) = oneshot::channel();
    //     log_info!("Старт запроса");
    //     // spawn_client_request_with_callback(tx, text.to_string());

    //     let bot_clone = bot.clone();
    //     tokio::spawn(async move {
    //         match rx.await {
    //             Ok(Ok(text)) => bot_clone.send_message(msg.chat.id, text).await.unwrap(),
    //             Ok(Err(e)) => bot_clone.send_message(msg.chat.id, e).await.unwrap(),
    //             Err(_) => bot_clone.send_message(msg.chat.id, "Ошибка связи с сервером").await.unwrap(),
    //         }
    //     });
    // }
    Ok(())
}