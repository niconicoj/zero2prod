use std::net::TcpListener;

use axum::{
    response::IntoResponse,
    routing::{get, IntoMakeService},
    Router,
};

use hyper::{server::conn::AddrIncoming, StatusCode};
use tracing::info;

pub type Server = axum::Server<AddrIncoming, IntoMakeService<Router>>;

#[derive(Default)]
pub struct ServerArgs {
    pub address: String,
}

pub fn server(listener: TcpListener) -> Server {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/health_check", get(health_check));

    let addr = listener
        .local_addr()
        .expect("Failed to get listener address");
    info!("listening on {}", addr);
    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
