mod proto {
    include!(concat!(env!("OUT_DIR"), "/prompt_service.rs"));
}

use proto::prompt_service_server::{PromptService, PromptServiceServer};
use proto::{PromptRequest, PromptResponse};
use tokio::fs;
use tonic::transport::{Certificate, Server, ServerTlsConfig};
use tonic::{Request, Response, Status};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use futures::StreamExt;
use std::pin::Pin;
use tonic::transport::Identity;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MyPromptService {
    tx: Arc<Mutex<Option<mpsc::Sender<PromptRequest>>>>,
}

impl Default for MyPromptService {
    fn default() -> Self {
        Self {
            tx: Arc::new(Mutex::new(None)),
        }
    }
}

#[tonic::async_trait]
impl PromptService for MyPromptService {
    type HandlePromptsStream =
        Pin<Box<dyn Stream<Item = Result<PromptRequest, Status>> + Send + 'static>>;

    async fn handle_prompts(
        &self,
        request: Request<tonic::Streaming<PromptResponse>>,
    ) -> Result<Response<Self::HandlePromptsStream>, Status> {
        println!("[SERVER] Клиент подключён");

        let (tx, rx) = mpsc::channel(128);

        // Сохраняем tx для последующего вызова send_prompt()
        self.tx.lock().unwrap().replace(tx);

        let mut client_responses = request.into_inner();

        // Задача для обработки ответов от клиента
        tokio::spawn({
            let tx_clone = Arc::clone(&self.tx);
            async move {
                while let Some(result) = client_responses.next().await {
                    match result {
                        Ok(response) => {
                            println!("[CLIENT RESPONDED]: {}", response.response_text);
                        }
                        Err(e) => {
                            println!("[ERROR FROM CLIENT]: {:?}", e);
                            break;
                        }
                    }
                }
                println!("[SERVER] Клиент отключён");
                tx_clone.lock().unwrap().take(); // Очищаем sender
            }
        });

        // Сервер не отправляет запрос автоматически, только по вызову send_prompt()
        let output_stream = ReceiverStream::new(rx).map(|msg| Ok(msg));
        Ok(Response::new(Box::pin(output_stream)))
    }
}

impl MyPromptService {
    /// Отправляет промпт клиенту
    pub fn send_prompt(&self, system_prompt: String, user_prompt: String) {
        if let Some(sender) = &*self.tx.lock().unwrap() {
            let _ = sender.blocking_send(PromptRequest {
                system_prompt,
                user_prompt,
            });
            println!("Запрос отправлен клиенту");
        } else {
            println!("Клиент не подключён. Невозможно отправить запрос.");
        }
    }
}

pub async fn start_grpc() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:5051".parse().unwrap();
    let service = MyPromptService::default();
    
    let service_clone = service.clone();
    // Function thats start another thread for gRPC crate
    tokio::spawn(async move {
        rustls::crypto::ring::default_provider().install_default().unwrap();

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

        println!("Запустили gRPC!");
        Server::builder()
            .tls_config(tls).unwrap()
            .add_service(PromptServiceServer::new(service_clone))
            .serve(addr)
            .await.unwrap();
    });

    Ok(())
}

// Получаем путь к папке certs в корне проекта
async fn get_certs_path() -> PathBuf {
    let path = std::env::current_dir().expect("Не могу получить текущий каталог");
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