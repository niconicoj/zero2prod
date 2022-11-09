use sea_orm::{ConnectionTrait, SqlxPostgresConnector};
use sea_orm::{DatabaseBackend, DatabaseConnection, DbErr, Statement};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tracing::error;

use crate::configuration::DatabaseSettings;

pub struct Db;

impl Db {
    #[tracing::instrument]
    pub fn init_db_connection(config: &DatabaseSettings) -> DatabaseConnection {
        Self::connect(config.with_db())
    }

    pub async fn create_database(config: &DatabaseSettings) -> Result<DatabaseConnection, DbErr> {
        let conn = Self::connect(config.without_db());
        match conn.get_database_backend() {
            DatabaseBackend::Postgres => {
                let result = conn
                    .execute(Statement::from_string(
                        conn.get_database_backend(),
                        format!("CREATE DATABASE \"{}\";", config.name),
                    ))
                    .await;

                match result {
                    Err(err) => match err {
                        DbErr::Exec(rt_err) => {
                            if !rt_err.contains("already exists") {
                                return Err(DbErr::Exec(rt_err));
                            }
                        }
                        _ => {
                            error!("{}", err);
                            return Err(err);
                        }
                    },
                    _ => {}
                };
            }
            _ => return Err(DbErr::Custom("db unsupported".to_string())),
        };
        let conn = Self::connect(config.with_db());
        Ok(conn)
    }

    fn connect(options: PgConnectOptions) -> DatabaseConnection {
        SqlxPostgresConnector::from_sqlx_postgres_pool(
            PgPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_secs(60))
                .connect_lazy_with(options),
        )
    }

    pub async fn wipe_database(settings: &DatabaseSettings) -> Result<(), DbErr> {
        let conn = Self::connect(settings.without_db());
        match conn.get_database_backend() {
            DatabaseBackend::Postgres => {
                conn.execute(Statement::from_string(
                    conn.get_database_backend(),
                    format!(
                        r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                AND pid <> pg_backend_pid();
                "#,
                        settings.name
                    ),
                ))
                .await?;
                conn.execute(Statement::from_string(
                    conn.get_database_backend(),
                    format!("DROP DATABASE \"{}\";", settings.name),
                ))
                .await?;
            }
            _ => {}
        };
        Ok(())
    }
}
