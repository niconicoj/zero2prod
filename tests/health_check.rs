use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let app_address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{app_address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");
    tokio::spawn(zero2prod::server(listener));
    address
}
