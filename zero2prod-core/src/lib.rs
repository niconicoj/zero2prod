use std::net::TcpListener;

use axum::{
    routing::{get, post, IntoMakeService},
    Router,
};

use hyper::server::conn::AddrIncoming;
use sqlx::PgPool;
use tracing::info;

use crate::{
    handlers::{health_check::health_check, subscriptions::subscribe},
    request_id::TraceIdLayer,
};

mod db;
mod error;
mod handlers;
mod request_id;

pub mod configuration;
pub mod telemetry;
pub mod testing;

pub type Server = axum::Server<AddrIncoming, IntoMakeService<Router>>;

#[derive(Default)]
pub struct ServerArgs {
    pub address: String,
}

pub fn server(listener: TcpListener, pool: PgPool) -> Server {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(pool)
        .layer(TraceIdLayer);

    let addr = listener
        .local_addr()
        .expect("Failed to get listener address");
    info!("listening on {}", addr);

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
}
