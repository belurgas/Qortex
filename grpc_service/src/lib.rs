pub mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));
}

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use tonic::{Request, Response, Status};

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
}
