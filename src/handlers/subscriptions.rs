use axum::{extract::rejection::FormRejection, Form};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::types::chrono::Utc;
use sqlx::types::Uuid;

use crate::db::DatabaseConnection;
use crate::error::{db_error, form_rejection};

#[derive(Deserialize)]
pub struct SubscribeRequest {
    name: String,
    email: String,
}

pub async fn subscribe(
    DatabaseConnection(mut conn): DatabaseConnection,
    form: Result<Form<SubscribeRequest>, FormRejection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let form = form.map_err(form_rejection)?;

    sqlx::query!(
        r#"
                INSERT INTO subscriptions (id, email, name, subscribed_at) 
                VALUES ($1, $2, $3, $4)
                "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&mut *conn)
    .await
    .map_err(db_error)?;
    Ok(StatusCode::OK)
}
