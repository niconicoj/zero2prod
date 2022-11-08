use sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, SqlxPostgresConnector, Statement,
};
use sea_orm_migration::prelude::*;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};

use crate::configuration::DatabaseSettings;

mod m20221022_142846_create_table_subscriptions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20221022_142846_create_table_subscriptions::Migration,
        )]
    }
}

pub async fn run_migrations(conn: &DatabaseConnection) -> Result<(), DbErr> {
    let pending_migrations = Migrator::get_pending_migrations(conn).await?;
    if pending_migrations.is_empty() {
        info!("migrations are up to date");
    } else {
        info!("running migrations");
        Migrator::up(conn, None).await?;
    }
    Ok(())
}

pub async fn create_database(
    conn: DatabaseConnection,
    db_settings: &DatabaseSettings,
) -> Result<DatabaseConnection, DbErr> {
    let conn = match conn.get_database_backend() {
        DatabaseBackend::MySql => return Err(DbErr::Custom("MySQL unsupported".to_string())),
        DatabaseBackend::Postgres => {
            let result = conn
                .execute(Statement::from_string(
                    conn.get_database_backend(),
                    format!("CREATE DATABASE \"{}\";", db_settings.name),
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

            let connection_pool = PgPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_secs(20))
                .connect_lazy_with(db_settings.with_db());

            SqlxPostgresConnector::from_sqlx_postgres_pool(connection_pool)
        }
        DatabaseBackend::Sqlite => conn,
    };
    Ok(conn)
}

pub async fn wipe_database(
    conn: &DatabaseConnection,
    db_settings: &DatabaseSettings,
) -> Result<(), DbErr> {
    match conn.get_database_backend() {
        DatabaseBackend::MySql => {
            conn.execute(Statement::from_string(
                conn.get_database_backend(),
                format!("DROP DATABASE `{}`;", db_settings.name),
            ))
            .await?;
        }
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
                    db_settings.name
                ),
            ))
            .await?;
            conn.execute(Statement::from_string(
                conn.get_database_backend(),
                format!("DROP DATABASE \"{}\";", db_settings.name),
            ))
            .await?;
        }
        DatabaseBackend::Sqlite => {}
    };
    Ok(())
}
