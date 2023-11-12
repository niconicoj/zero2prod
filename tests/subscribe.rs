use std::net::TcpListener;

use hyper::StatusCode;

use sqlx::{Connection, PgConnection};
use zero2prod::configuration;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app();

    let client = reqwest::Client::new();

    let body = "name=John%20Doe&email=john.doe@gmail.com";
    let response = client
        .post(&format!("{app_address}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let configuration = configuration::get_configuration().expect("Failed to read configuration.");
    let mut connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "john.doe@gmail.com");
    assert_eq!(saved.name, "John Doe");
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_form_data() {
    let app_address = spawn_app();

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=John%20Doe", "missing the email"),
        ("email=john.doe@gmail.com", "missing the name"),
        ("", "empty"),
    ];

    for (body, error) in test_cases {
        let response = client
            .post(&format!("{app_address}/subscriptions"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Expected a 400 Bad Request when the payload was {} but got {} instead",
            error,
            response.status()
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");
    tokio::spawn(zero2prod::server(listener));
    address
}
