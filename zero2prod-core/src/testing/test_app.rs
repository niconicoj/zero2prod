use std::{future::Future, panic, pin::Pin};

use crate::{
    configuration::{self, Configuration, WithDb},
    server,
    telemetry::setup_subscriber,
    Address,
};

use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        setup_subscriber("integration-test", "debug", std::io::stdout);
    } else {
        setup_subscriber("integration-test", "debug", std::io::sink);
    }
});

#[derive(Clone)]
pub struct TestApp {
    pub config: Configuration,
    pub address: Address,
    pub pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let config = configuration::get_test_configuration().expect("Failed to read configuration");

    let (server, address, pool) = server(&config);
    configure_database(&config).await;

    tokio::spawn(server);
    TestApp {
        config,
        address,
        pool,
    }
}

pub async fn configure_database(configuration: &Configuration) -> (String, PgPool) {
    let mut conn = PgConnection::connect(
        configuration
            .db
            .connection_string(WithDb::No)
            .expose_secret(),
    )
    .await
    .expect("Failed to connect to Postgres");

    println!("Creating database: {}", configuration.db.name);
    conn.execute(format!(r#"CREATE DATABASE "{}";"#, configuration.db.name).as_str())
        .await
        .unwrap();

    let db_pool = PgPool::connect(
        configuration
            .db
            .connection_string(WithDb::Yes)
            .expose_secret(),
    )
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
    let mut conn = PgConnection::connect(
        test_app
            .config
            .db
            .connection_string(WithDb::No)
            .expose_secret(),
    )
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
    T: FnOnce(TestApp) -> Pin<Box<dyn Future<Output = ()> + 'static + Send>>,
{
    let result = std::panic::catch_unwind(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let test_app = spawn_app().await;

                test(test_app.clone()).await;

                teardown_app(test_app).await;
            })
    });

    assert!(result.is_ok());
}
