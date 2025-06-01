use proto::{user_service_server::UserService, GetAllUsersRequest, GetAllUsersResponse, User};
use tonic::{transport::Server, Request, Response, Status};

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/ai_service.rs"));
}

#[derive(Debug, Default)]
pub struct MyUserSevice {}

#[tonic::async_trait]
impl UserService for MyUserSevice {
    async fn get_all_users(
        &self,
        _request: Request<GetAllUsersRequest>,
    ) -> Result<Response<GetAllUsersResponse>, Status> {
        // Тут получаем данные из бд
        // Пока используем заглушку
        let users = vec![
            User {
                id: "1".into(),
                name: "Alice".into(),
                email: "alice@example.com".into(),
            },
            User {
                id: "2".into(),
                name: "Bob".into(),
                email: "bob@example.com".into(),
            },
        ];

        Ok(Response::new(GetAllUsersResponse { users }))
    }
}