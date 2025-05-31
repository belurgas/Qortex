pub mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));
}

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest, ClientMessage, ServerMessage};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use std::pin::Pin;
use futures::{Stream, StreamExt};

/// Struct for Greeter impl. For test view...
#[derive(Debug, Default)]
pub struct MyGreeter {}

/// When the client send SayHello method gRPC server reply for his request
/// The methos discrive in hellowrorld.proto
#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Получен запрос: {:?}", request);

        // Reply for request
        let reply = HelloReply {
            message: format!("Привет {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }

    /// Тип для исходящего потока сообщений
    type ChatStreamStream = Pin<Box<dyn Stream<Item = Result<ServerMessage, Status>> + Send>>;
    
    // Двунапраленный streaming RPC метод
    async fn chat_stream(
        &self,
        request: Request<tonic::Streaming<ClientMessage>>
    ) -> Result<Response<Self::ChatStreamStream>, Status> {
        println!("[STREAMING] Установлено");

        // Канал для обмена между задачами
        let (tx, rx) = mpsc::channel(128);

        let mut incoming_stream = request.into_inner();

        // задача для обработки входящих
        tokio::spawn(async move {
            while let Some(message) = incoming_stream.next().await {
                match message {
                    Ok(msg) => {
                        println!("[STREAMING] Получено сообщение: {}", msg.text);

                        let response = ServerMessage {
                            text: format!("ВВ: {}", msg.text),
                        };

                        if let Err(e) = tx.send(Ok(response)).await {
                            eprintln!("[STREAMING] Ошибка отправки: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("[STREAMING] Ошибка в потоке: {:?}", e);
                        break;
                    }
                }
            }
            println!("[STREAMING] Клиент отключился");
        });

        let output_stream = ReceiverStream::new(rx);

        Ok(Response::new(
            Box::pin(output_stream) as Self::ChatStreamStream
        ))
    }
}
