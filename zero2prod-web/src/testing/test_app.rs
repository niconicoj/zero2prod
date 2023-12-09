use std::{future::Future, panic, pin::Pin};

use crate::{
    configuration::{self, Configuration, WithDb},
    server::{self, Address},
    telemetry::setup_subscriber,
};

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use wiremock::MockServer;

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        setup_subscriber("integration-test", "debug", std::io::stdout);
    } else {
        setup_subscriber("integration-test", "debug", std::io::sink);
    }
});

pub struct TestStack {
    pub app: TestApp,
    pub email_server: MockServer,
}

#[derive(Clone)]
pub struct TestApp {
    pub config: Configuration,
    pub address: Address,
    pub pool: PgPool,
}

pub async fn spawn_app() -> (TestApp, MockServer) {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let mut config = configuration::get_test_configuration().expect("Failed to read configuration");
    config.email_client.base_url = email_server.uri();

    let (server, address, pool) = server::start(&config).await;
    configure_database(&config).await;

    tokio::spawn(async { server.await.unwrap() });
    (
        TestApp {
            config,
            address,
            pool,
        },
        email_server,
    )
}

pub async fn configure_database(configuration: &Configuration) -> (String, PgPool) {
    let mut conn = PgConnection::connect_with(&configuration.db.connection_options(WithDb::No))
        .await
        .expect("Failed to connect to Postgres");

    println!("Creating database: {}", configuration.db.name);
    conn.execute(format!(r#"CREATE DATABASE "{}";"#, configuration.db.name).as_str())
        .await
        .unwrap();

    let db_pool = PgPool::connect_with(configuration.db.connection_options(WithDb::Yes))
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("../migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate database");

    (configuration.db.name.clone(), db_pool)
}

async fn teardown_app(test_app: TestApp) {
    test_app.pool.close().await;
    let mut conn = PgConnection::connect_with(&test_app.config.db.connection_options(WithDb::No))
        .await
        .expect("Failed to connect to Postgres");

    conn.execute(
        format!(
            r#"
        SELECT pg_terminate_backend(pg_stat_activity.pid)
        FROM pg_stat_activity
        WHERE pg_stat_activity.datname = '{}';
    "#,
            test_app.config.db.name
        )
        .as_str(),
    )
    .await
    .unwrap();

    conn.execute(format!(r#"DROP DATABASE "{}";"#, test_app.config.db.name).as_str())
        .await
        .unwrap();
}

pub fn run_test<T>(test: T)
where
    T: panic::UnwindSafe,
    T: FnOnce(TestStack) -> Pin<Box<dyn Future<Output = ()> + 'static + Send>>,
{
    let result = std::panic::catch_unwind(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let (test_app, email_server) = spawn_app().await;

                test(TestStack {
                    app: test_app.clone(),
                    email_server,
                })
                .await;

                teardown_app(test_app).await;
            })
    });

    assert!(result.is_ok());
}
