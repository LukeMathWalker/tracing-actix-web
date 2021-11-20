pub mod domain;
pub mod init;
pub mod protocol;
use std::net::SocketAddr;

use protocol::users::protocol_server::Protocol;
use protocol::users::protocol_server::ProtocolServer;

use init::once;
use tonic::{transport::Server, Response, Status};
use tracing_actix_web::RelayService;

#[derive(Default)]
pub struct Service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    once::init();
    let address: SocketAddr = "127.0.0.1:50051".parse()?;
    let service = Service::default();
    Server::builder()
        .add_service(RelayService::new(ProtocolServer::new(service)))
        .serve(address)
        .await?;
    Ok(())
}

#[tonic::async_trait]
impl Protocol for Service {
    #[tracing::instrument(name = "Fetch users", skip(self), fields(protocol = "grpc"))]
    async fn fetch(
        &self,
        request: tonic::Request<protocol::users::Filter>,
    ) -> Result<Response<protocol::users::Users>, Status> {
        let database = vec![
            protocol::users::User {
                name: "John".to_string(),
                gender: protocol::users::Gender::Male as i32,
            },
            protocol::users::User {
                name: "Bob".to_string(),
                gender: protocol::users::Gender::Male as i32,
            },
            protocol::users::User {
                name: "Alice".to_string(),
                gender: protocol::users::Gender::Female as i32,
            },
        ];
        let filter = request.into_inner();
        let users = database
            .into_iter()
            .filter(|user| {
                filter
                    .gender
                    .map(|gender| user.gender == gender)
                    .unwrap_or(true)
            })
            .collect();
        Ok(tonic::Response::new(protocol::users::Users { users }))
    }
}
