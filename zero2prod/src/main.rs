use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use zero2prod_core::configuration::{get_configuration, WithDb};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let configuration = get_configuration(None).expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.port);
    let listener = TcpListener::bind(&address).expect("Failed to bind listener");

    info!("Setting up database connection pool");
    let pool = PgPoolOptions::new()
        .connect(&configuration.database.connection_string(WithDb::Yes))
        .await
        .expect("Failed to connect to database");

    info!("Starting server");

    zero2prod_core::server(listener, pool).await.unwrap()
}
