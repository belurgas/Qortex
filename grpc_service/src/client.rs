use std::fmt::Error;

use logging::{log_error, log_info};
use tonic::Request;
use tokio::sync::oneshot;

use crate::server::proto::{ai_generation_service_client::AiGenerationServiceClient, TextGenerationRequest};

pub fn spawn_client_request_with_callback(
    sender: oneshot::Sender<Result<String, String>>,
    text: String,
) {
    tokio::spawn(async move {
        let result = async move {
            let mut client = AiGenerationServiceClient::connect("http://127.0.0.1:50052").await.unwrap();

            let request = Request::new(TextGenerationRequest {
                system_prompt: "You are a helpful assistant.".to_string(),
                user_prompt: text,
                temperature: 0.7,
                top_p: 0.9,
            });

            let response = client.generate_text(request).await.unwrap();
            Ok::<_, String>(response.into_inner().generated_text)
        }
        .await;

        let _ = sender.send(result);
    });
}