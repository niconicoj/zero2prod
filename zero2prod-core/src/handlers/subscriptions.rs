use std::sync::Arc;

use axum::Extension;
use axum::{extract::rejection::FormRejection, Form};
use hyper::StatusCode;

use sqlx::types::chrono::Utc;
use sqlx::types::Uuid;

use tracing::{info, instrument, Span};

use crate::db::DatabaseConnection;
use crate::domain::NewSubscriber;
use crate::email::client::EmailClient;
use crate::error::{db_error, form_rejection, internal_error};

#[instrument(name = "Adding a new subscriber", skip_all)]
pub async fn subscribe(
    DatabaseConnection(mut conn): DatabaseConnection,
    Extension(email_client): Extension<Arc<EmailClient>>,
    form: Result<Form<NewSubscriber>, FormRejection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let form = form.map_err(form_rejection)?;

    Span::current()
        .record("subscriber_email", form.email.as_ref())
        .record("subscriber_name", form.name.as_ref());

    info!("Adding a new subscriber : {}", form.email);
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status) 
            VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        form.email.as_ref(),
        form.name.as_ref(),
        Utc::now()
    )
    .execute(&mut *conn)
    .await
    .map_err(db_error)?;

    info!("Sending confirmation email to {}", form.email);
    email_client
        .send_email(&form.email, "Hello", "<p>Hello world</p>", "")
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::OK)
}
