use axum::{extract::rejection::FormRejection, Form};
use hyper::StatusCode;
use serde::Deserialize;

use sqlx::types::chrono::Utc;
use sqlx::types::Uuid;

use tracing::{info, instrument, Span};

use crate::db::DatabaseConnection;
use crate::error::{db_error, form_rejection};
use crate::request_id::RequestId;

#[derive(Deserialize)]
pub struct SubscribeRequest {
    name: String,
    email: String,
}

#[instrument(name = "Adding a new subscriber", skip(conn, form))]
pub async fn subscribe(
    DatabaseConnection(mut conn): DatabaseConnection,
    RequestId(request_id): RequestId,
    form: Result<Form<SubscribeRequest>, FormRejection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let form = form.map_err(form_rejection)?;

    Span::current()
        .record("subscriber_email", &form.email)
        .record("subscriber_name", &form.name);

    info!("Adding a new subscriber : {}", form.email);
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
