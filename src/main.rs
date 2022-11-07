use clap::Parser;
use std::process::exit;

use tracing::{error, info};
use zero2prod::{
    configuration::get_configuration,
    run,
    telemetry::{get_subscriber, init_subscriber},
};

use crate::cli::Cli;

mod cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let configuration = match get_configuration(&cli.profiles) {
        Ok(config) => config,
        Err(_) => {
            error!("failed to read configuration");
            exit(1);
        }
    };

    let subscriber = get_subscriber(&configuration, std::io::stdout);
    init_subscriber(subscriber);

    if cli.profiles.len() > 0 {
        info!("active profiles : {}", cli.profiles.join(","));
    }
    info!("log filter : {}", &configuration.env_filter);

    let server = run(&configuration).await.unwrap();
    info!(
        "accepting connection at {}",
        server.local_addr().to_string()
    );
    server.await.unwrap();
}
