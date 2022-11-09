use clap::Parser;
use std::process::exit;
use tokio::join;

use tracing::{error, info};
use zero2prod::{
    configuration::get_configuration,
    migrations, run,
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

    let (server, conn) = run(&configuration).await.unwrap();
    info!(
        "accepting connection at {}",
        server.local_addr().to_string()
    );

    let (srv_result, db_res) = join!(server, migrations::run_migrations(&conn));
    db_res.unwrap();
    srv_result.unwrap();
}
