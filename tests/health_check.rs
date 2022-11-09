use hyper::StatusCode;
use sea_orm::prelude::Uuid;
use sea_orm::prelude::*;
use tokio::sync::OnceCell;
use zero2prod::configuration::{get_configuration, Settings};

use zero2prod::db::Db;
use zero2prod::entities::prelude::*;
use zero2prod::migrations;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

struct TestApp {
    pub url: String,
    pub db: DatabaseConnection,
    pub configuration: Settings,
}

static ONCE: OnceCell<()> = OnceCell::const_new();

async fn tracing(configuration: &Settings) {
    match configuration.test_log {
        Some(true) => {
            let subscriber = get_subscriber(&configuration, std::io::stdout);
            init_subscriber(subscriber);
        }
        _ => {
            let subscriber = get_subscriber(&configuration, std::io::sink);
            init_subscriber(subscriber);
        }
    };
}

impl TestApp {
    async fn spawn() -> Result<Self, DbErr> {
        let mut configuration = get_configuration(&vec!["test".to_string()]).unwrap();
        ONCE.get_or_init(|| tracing(&configuration)).await;

        configuration.database.name = Uuid::new_v4().to_string();
        let db = Db::create_database(&configuration.database).await?;

        let (server, conn) = zero2prod::run(&configuration).await.unwrap();
        migrations::run_migrations(&conn).await?;
        let url = format!("http://{}", server.local_addr());
        let _ = tokio::spawn(server);

        println!(
            "spawned app => url: {}, db: {}",
            url, configuration.database.name
        );
        Ok(Self {
            url,
            db,
            configuration,
        })
    }

    async fn clean(&mut self) -> Result<(), DbErr> {
        Db::wipe_database(&self.configuration.database).await?;
        Ok(())
    }
}

#[tokio::test]
async fn health_check_works() {
    let mut app = TestApp::spawn().await.unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", app.url))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(Some(0), response.content_length());
    app.clean().await.unwrap();
}

#[tokio::test]
async fn given_valid_form_data_expect_subscribe_return_200() {
    let mut app = TestApp::spawn().await.unwrap();

    let client = reqwest::Client::new();

    let body = r#"{
            "name": "John Doe",
            "email": "john.doe@gmail.com"
        }"#;

    let response = client
        .post(format!("{}/subscriptions", app.url))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .expect("fail to execute request");

    assert_eq!(StatusCode::CREATED, response.status());

    let subscriber = Subscription::find()
        .one(&app.db)
        .await
        .expect("failed to query db")
        .expect("the subscriber was not saved in db");

    assert_eq!(subscriber.name, "John Doe");
    assert_eq!(subscriber.email, "john.doe@gmail.com");
    app.clean().await.unwrap();
}

#[tokio::test]
async fn given_invalid_form_data_expect_subscribe_return_400() {
    let mut app = TestApp::spawn().await.unwrap();

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("{\"name\": \"John Doe\"}", "missing the email"),
        ("{\"email\": \"john.doe@gmail.com\"}", "missing the name"),
        ("{}", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app.url))
            .header("Content-Type", "application/json")
            .body(invalid_body)
            .send()
            .await
            .expect("fail to execute request");

        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            "did not emit 400 given {}",
            error_message
        );
    }
    app.clean().await.unwrap();
}
