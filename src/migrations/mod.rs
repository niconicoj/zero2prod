use sea_orm::DatabaseConnection;
use sea_orm_migration::prelude::*;
use tracing::info;

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
