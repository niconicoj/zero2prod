use std::{future::Future, net::TcpListener, panic, pin::Pin};

use crate::configuration::{Configuration, WithDb};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

pub struct TestApp {
    pub app_address: String,
    pub db_pool: PgPool,
    pub db_name: String,
    pub db_address: String,
}

pub async fn spawn_app() -> (TestApp, String, String) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
    let port = listener.local_addr().unwrap().port();
    let app_address = format!("http://127.0.0.1:{port}");

    let mut configuration = crate::configuration::get_configuration(Some("../configuration.yaml"))
        .expect("Failed to read configuration");
    configuration.database.database_name = format!(
        "{}-{}",
        configuration.database.database_name,
        Uuid::new_v4()
    );

    let (test_database_name, db_pool) = configure_database(&configuration).await;

    tokio::spawn(crate::server(listener, db_pool.clone()));
    (
        TestApp {
            app_address,
            db_pool,
            db_name: test_database_name.clone(),
            db_address: configuration.database.connection_string(WithDb::No),
        },
        test_database_name.clone(),
        configuration.database.connection_string(WithDb::No),
    )
}

pub async fn configure_database(configuration: &Configuration) -> (String, PgPool) {
    let mut conn = PgConnection::connect(&configuration.database.connection_string(WithDb::No))
        .await
        .expect("Failed to connect to Postgres");

    conn.execute(
        format!(
            r#"CREATE DATABASE "{}";"#,
            configuration.database.database_name
        )
        .as_str(),
    )
    .await
    .unwrap();

    let db_pool = PgPool::connect(&configuration.database.connection_string(WithDb::Yes))
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("../migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate database");

    (configuration.database.database_name.clone(), db_pool)
}

async fn drop_database(test_database_name: String, db_conn_string: String) {
    let mut conn = PgConnection::connect(&db_conn_string)
        .await
        .expect("Failed to connect to Postgres");

    conn.execute(
        format!(
            r#"
        SELECT pg_terminate_backend(pg_stat_activity.pid)
        FROM pg_stat_activity
        WHERE pg_stat_activity.datname = '{}';
    "#,
            test_database_name
        )
        .as_str(),
    )
    .await
    .unwrap();

    conn.execute(format!(r#"DROP DATABASE "{}";"#, test_database_name).as_str())
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
                let (test_app, test_database_name, db_conn_string) = spawn_app().await;
                let pool = test_app.db_pool.clone();

                test(test_app).await;

                pool.close().await;
                drop_database(test_database_name, db_conn_string).await;
            })
    });

    assert!(result.is_ok());
}
