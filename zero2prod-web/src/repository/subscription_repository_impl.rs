use async_trait::async_trait;
use sqlx::types::chrono::Utc;
use sqlx::PgPool;

use zero2prod_core::{
    domain::NewSubscriber,
    error::{CoreError, CoreResult},
    repository::SubscriptionRepository,
};

use uuid::Uuid;

pub struct SubscriptionRepositoryImpl {
    db_pool: PgPool,
}

impl SubscriptionRepositoryImpl {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl SubscriptionRepository for SubscriptionRepositoryImpl {
    async fn create(&self, new_subscriber: &NewSubscriber) -> CoreResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status) 
            VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
            Uuid::new_v4(),
            new_subscriber.email.as_ref(),
            new_subscriber.name.as_ref(),
            Utc::now()
        )
        .execute(&self.db_pool)
        .await
        .map_err(db_error)?;

        Ok(())
    }
}

pub fn db_error(err: sqlx::Error) -> CoreError {
    match err {
        sqlx::Error::Database(err) if err.is_unique_violation() => CoreError::EmailAlreadyExists,
        err => CoreError::Unexpected(err.to_string()),
    }
}
