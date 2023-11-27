use tracing::info;
use zero2prod_core::{configuration::get_configuration, server};

#[tokio::main]
async fn main() {
    zero2prod_core::telemetry::setup_subscriber(
        "zero2prod",
        "zero2prod_core::request_id=trace,info",
        std::io::stdout,
    );

    let configuration = get_configuration().expect("Failed to read configuration.");
    info!("Active profile : {}", configuration.profile);

    let (server, _, pool) = server::start(&configuration).await;

    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database.");

    server.await.unwrap();
}
