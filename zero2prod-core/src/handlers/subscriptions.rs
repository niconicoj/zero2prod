use std::sync::Arc;

use axum::Extension;
use axum::{extract::rejection::FormRejection, Form};
use hyper::StatusCode;

use sqlx::types::chrono::Utc;
use sqlx::types::Uuid;

use tracing::{info, instrument, Span};

use crate::configuration::Configuration;
use crate::db::DatabaseConnection;
use crate::domain::NewSubscriber;
use crate::email::client::EmailClient;
use crate::error::{db_error, form_rejection, internal_error};
use crate::templates::{Template, TemplateEngine};

#[instrument(name = "Adding a new subscriber", skip_all)]
pub async fn subscribe(
    DatabaseConnection(mut conn): DatabaseConnection,
    Extension(email_client): Extension<Arc<EmailClient>>,
    Extension(config): Extension<Arc<Configuration>>,
    Extension(template_engine): Extension<TemplateEngine>,
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
    let confirmation_link = format!(
        "http://{}:{}/subscriptions/confirm",
        config.app.host, config.app.port
    );

    email_client
        .send_email(
            &form.email,
            "Hello",
            &template_engine.render(Template::ConfirmationHtml {
                link: &confirmation_link,
            }),
            &template_engine.render(Template::ConfirmationTxt {
                link: &confirmation_link,
            }),
        )
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::OK)
}
