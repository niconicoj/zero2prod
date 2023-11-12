use std::net::TcpListener;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::runtime::Handle;
use uuid::Uuid;
use zero2prod::configuration::{Configuration, WithDb};

pub struct TestApp {
    pub address: String,
    db_name: String,
    pub db_pool: PgPool,
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let handle = Handle::current();
        let db_name = self.db_name.clone();
        let pool = self.db_pool.clone();
        std::thread::spawn(move || {
            handle.block_on(delete_test_db(pool, db_name));
        });
    }
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let mut configuration =
        zero2prod::configuration::get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = format!(
        "{}-{}",
        configuration.database.database_name,
        Uuid::new_v4()
    );

    let db_pool = configure_database(&configuration).await;

    tokio::spawn(zero2prod::server(listener, db_pool.clone()));
    TestApp {
        address,
        db_pool,
        db_name: configuration.database.database_name,
    }
}

pub async fn configure_database(configuration: &Configuration) -> PgPool {
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

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate database");

    db_pool
}

async fn delete_test_db(pg_pool: PgPool, db_name: String) {
    sqlx::query(&format!(r#"DROP DATABASE "{}";"#, db_name))
        .execute(&pg_pool)
        .await
        .expect("Failed to delete test database");
}
