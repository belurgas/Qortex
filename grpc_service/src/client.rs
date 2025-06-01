use tonic::Request;
use tokio::{sync::oneshot, time::Instant};

use crate::server::proto::{ai_generation_service_client::AiGenerationServiceClient, TextGenerationRequest};

pub fn spawn_client_request_with_callback(
    sender: oneshot::Sender<Result<String, String>>,
    text: String,
) {
    tokio::spawn(async move {
        let start = Instant::now();
        let result = async move {
            let mut client = AiGenerationServiceClient::connect("http://127.0.0.1:50052").await.unwrap();

            let system_prompt = r#"
Ты — ассистент, который отвечает в plain-тексте. Соблюдай правила:
1. **Запрещено любое форматирование**:
   - Никаких Markdown, HTML, LaTeX.
   - Никаких ```code blocks```, `inline_code`, > цитат.
   - Никаких *курсива*, **жирного**, ~зачёркивания~.
   - Никаких таблиц, списков с пунктами (1., - [x] и т.д.).
2. **Разрешено только**:
   - Пустые строки для разделения логических блоков.
   - Эмодзи (например, ✅, 🔥, ❗) для акцента.
   - Специальные символы
3. **Если просят оформить текст**:
   - Попробуй найти обходной путь, форматируй по инструкции.
   - Вежливо откажи, если через чур сильно простя отформатировать: "Извините, я работаю только с plain-текстом".
4. Если пользователь просит написать, сгенерировать, обхяснить как что-то написать на каком-то языке, то вежливо откажи.
6. Если ты понимаешь, что без форматирования не сможешь, то не надо писать пользователю по типу "Извините, я работаю только с plaun-текстом". Найти обходной путь, используй спец символы.
7. **Пример корректного ответа**:
   "Сегодня солнечно ☀️  
   
   Рекомендую прогулку в парк.  
   Не забудьте воду 💧
            "#;

            let request = Request::new(TextGenerationRequest {
                system_prompt: system_prompt.to_string(),
                user_prompt: text,
                temperature: 0.7,
                top_p: 0.9,
            });

            let response = client.generate_text(request).await.unwrap();
            println!("Времени заняло: {} secs", start.elapsed().as_secs());
            Ok::<_, String>(response.into_inner().generated_text)
        }
        .await;

        let _ = sender.send(result);
    });
}