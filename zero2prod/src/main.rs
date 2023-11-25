use tracing::info;
use zero2prod_core::configuration::get_configuration;

#[tokio::main]
async fn main() {
    zero2prod_core::telemetry::setup_subscriber(
        "zero2prod",
        "zero2prod_core::request_id=trace,info",
        std::io::stdout,
    );

    let configuration = get_configuration().expect("Failed to read configuration.");
    info!("Active profile : {}", configuration.profile);

    let (server, address, _) = zero2prod_core::server(&configuration);

    info!("Listening on {}", address);
    server.await.unwrap();
}
