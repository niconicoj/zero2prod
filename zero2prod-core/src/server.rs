use std::fmt::{self, Display, Formatter};

use axum::{
    routing::{get, post, IntoMakeService},
    serve::Serve,
    Router,
};

use crate::{configuration::Configuration, email};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

use tokio::net::TcpListener;

use crate::{
    configuration::WithDb,
    handlers::{health_check::health_check, subscriptions::subscribe},
    request_id::TraceIdLayer,
};

#[derive(Default, Clone)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

pub type Server = Serve<IntoMakeService<Router>, Router>;

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "http://{}:{}", self.host, self.port)
    }
}

pub async fn start(configuration: &Configuration) -> (Server, Address, PgPool) {
    let address = format!("{}:{}", configuration.app.host, configuration.app.port);
    let listener = TcpListener::bind(address)
        .await
        .expect("Failed to bind listener");

    info!("Setting up database connection pool");
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(configuration.db.timeout))
        .connect_lazy_with(configuration.db.connection_options(WithDb::Yes));

    let email_client = email::client::EmailClient::from_config(&configuration.email_client);

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(pool.clone())
        .with_state(email_client.clone())
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
        axum::serve(listener, app.into_make_service()),
        app_address,
        pool,
    )
}
