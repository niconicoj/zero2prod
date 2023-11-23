use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use zero2prod_core::configuration::{get_configuration, WithDb};

#[tokio::main]
async fn main() {
    zero2prod_core::telemetry::setup_subscriber(
        "zero2prod",
        "zero2prod_core::request_id=trace,info",
        std::io::stdout,
    );

    let configuration = get_configuration(None).expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.port);
    let listener = TcpListener::bind(&address).expect("Failed to bind listener");

    info!("Setting up database connection pool");
    let pool = PgPoolOptions::new()
        .connect(
            configuration
                .database
                .connection_string(WithDb::Yes)
                .expose_secret(),
        )
        .await
        .expect("Failed to connect to database");

    info!("Starting server");

    zero2prod_core::server(listener, pool).await.unwrap()
}
