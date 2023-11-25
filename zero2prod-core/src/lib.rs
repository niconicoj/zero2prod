use std::{
    fmt::{self, Display, Formatter},
    net::TcpListener,
};

use axum::{
    routing::{get, post, IntoMakeService},
    Router,
};

use configuration::Configuration;
use hyper::server::conn::AddrIncoming;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

use crate::{
    configuration::WithDb,
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

#[derive(Default, Clone)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "http://{}:{}", self.host, self.port)
    }
}

pub fn server(configuration: &Configuration) -> (Server, Address, PgPool) {
    let address = format!("{}:{}", configuration.app.host, configuration.app.port);
    let listener = TcpListener::bind(address).expect("Failed to bind listener");

    info!("Setting up database connection pool");
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(
            configuration.db.timeout.unwrap_or(2),
        ))
        .connect_lazy_with(configuration.db.connection_options(WithDb::Yes));

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(pool.clone())
        .layer(TraceIdLayer);

    let app_address = Address {
        host: listener.local_addr().unwrap().ip().to_string(),
        port: listener.local_addr().unwrap().port(),
    };

    let addr = listener
        .local_addr()
        .expect("Failed to get listener address");

    info!("listening on {}", addr);
    (
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service()),
        app_address,
        pool,
    )
}
