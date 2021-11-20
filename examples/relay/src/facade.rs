pub mod domain;
pub mod init;
pub mod protocol;
use protocol::users::protocol_client::ProtocolClient;

use actix_web::{
    get,
    http::Uri,
    web::{self, Query},
    App, HttpResponse, HttpServer, Responder,
};
use init::once;
use std::net::SocketAddr;
use tonic::{codegen::InterceptedService, transport::Channel};
use tracing_actix_web::{Relay, RelayInterceptor, TracingLogger};

pub type GrpcClient = ProtocolClient<InterceptedService<Channel, RelayInterceptor>>;

struct State {
    client: GrpcClient,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    once::init();
    let address: SocketAddr = "127.0.0.1:8080".parse()?;
    let uri: Uri = "http://127.0.0.1:50051".parse()?;
    let channel = Channel::builder(uri).connect().await?;
    let client: GrpcClient = ProtocolClient::with_interceptor(channel, RelayInterceptor);
    HttpServer::new(move || {
        App::new()
            .wrap(Relay)
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(State {
                client: client.clone(),
            }))
            .configure(|config| {
                config.service(fetch);
            })
    })
    .bind(address)?
    .run()
    .await?;
    Ok(())
}

#[tracing::instrument(name = "Fetch users", skip(state), fields(protocol = "rest"))]
#[get("/users")]
async fn fetch(state: web::Data<State>, filter: Option<Query<domain::Filter>>) -> impl Responder {
    use std::convert::TryFrom;
    let mut client: GrpcClient = state.client.clone();
    match client.fetch(protocol::users::Filter::from(filter)).await {
        Ok(response) => match Vec::<domain::User>::try_from(response.into_inner()) {
            Ok(users) => HttpResponse::Ok().json(users),
            Err(e) => {
                tracing::event!(
                    tracing::Level::ERROR,
                    "Failed to convert users from protobuf message: {:?}",
                    e
                );
                HttpResponse::InternalServerError().finish()
            }
        },
        Err(e) => {
            tracing::event!(tracing::Level::ERROR, "Failed to fetch users: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
