use std::net::TcpListener;

use zero2prod::server;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind listener");

    server(listener).await.unwrap()
}
