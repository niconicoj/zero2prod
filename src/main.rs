use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::{get_configuration, WithDb},
    server,
};

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.port);
    let listener = TcpListener::bind(&address).expect("Failed to bind listener");

    let pool = PgPoolOptions::new()
        .connect(&configuration.database.connection_string(WithDb::Yes))
        .await
        .expect("Failed to connect to database");

    server(listener, pool).await.unwrap()
}
