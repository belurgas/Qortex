// src/mistral.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
pub struct QueryMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct MistralRequest {
    pub temperature: f32,
    pub model: String,
    pub messages: Vec<QueryMessage>,
}

#[derive(Deserialize)]
struct Choice {
    message: MistralResponse,
}

#[derive(Deserialize)]
pub struct MistralResponse {
    pub content: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    pub choices: Vec<Choice>,
}

pub async fn query_mistral_api(input_text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = env::var("MISTRAL_API_KEY")?;

    let client = Client::new();

    let system_prompt = r#"
Ты — ассистент, который отвечает строго в plain-тексте. Соблюдай правила:
1. **Запрещено любое форматирование**:
   - Никаких Markdown, HTML, LaTeX.
   - Никаких ```code blocks```, `inline_code`, > цитат.
   - Никаких *курсива*, **жирного**, ~зачёркивания~.
   - Никаких таблиц, списков с пунктами (1., - [x] и т.д.).
2. **Разрешено только**:
   - Пустые строки для разделения логических блоков.
   - Эмодзи (например, ✅, 🔥, ❗) для акцента.
3. **Если просят оформить текст**:
   - Вежливо откажи: "Извините, я работаю только с plain-текстом".
4. Если пользователь просит написать, сгенерировать, обхяснить как что-то написать на каком-то языке, то вежливо откажи.
5. **Пример корректного ответа**:
   "Сегодня солнечно ☀️  
   
   Рекомендую прогулку в парк.  
   Не забудьте воду 💧""#;

    let body = MistralRequest {
        model: "pixtral-large-latest".to_string(),
        temperature: 0.3,
        messages: vec![
            QueryMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            QueryMessage {
                role: "user".to_string(),
                content: input_text.to_string(),
            },
        ],
    };

    let response = client
        .post("https://api.mistral.ai/v1/chat/completions") 
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?;

    let result: ApiResponse = response.json().await?;

    if let Some(choice) = result.choices.get(0) {
        Ok(choice.message.content.clone())
    } else {
        Err("No choices in the response".into())
    }
}