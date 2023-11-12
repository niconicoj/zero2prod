use std::net::TcpListener;

use zero2prod::{configuration::get_configuration, server};

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.port);
    let listener = TcpListener::bind(&address).expect("Failed to bind listener");

    server(listener).await.unwrap()
}
