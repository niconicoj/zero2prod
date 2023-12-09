use std::sync::Arc;

use axum::Extension;
use axum::{extract::rejection::FormRejection, Form};
use hyper::StatusCode;

use sqlx::types::Uuid;

use zero2prod_core::domain::{Document, DocumentKind, NewSubscriber};

use crate::configuration::Configuration;
use crate::error::{core_error, form_rejection};
use crate::repository::SubscriptionRepositoryImpl;
use crate::service::EmailServiceImpl;

pub async fn subscribe(
    Extension(config): Extension<Arc<Configuration>>,
    Extension(subscription_repository): Extension<Arc<SubscriptionRepositoryImpl>>,
    Extension(email_client): Extension<Arc<EmailServiceImpl>>,
    form: Result<Form<NewSubscriber>, FormRejection>,
) -> Result<StatusCode, (StatusCode, String)> {
    let form = form.map_err(form_rejection)?;

    let confirmation_link = format!(
        "http://{}/subscriptions/confirm?sub_id={}",
        config.app.host,
        Uuid::new_v4()
    );

    let confirmation_email = Document::new(
        "Welcome !".into(),
        DocumentKind::Confirmation { confirmation_link },
    );

    zero2prod_core::handlers::subscribe(
        subscription_repository.as_ref(),
        email_client.as_ref(),
        form.0,
        confirmation_email,
    )
    .await
    .map_err(core_error)?;

    Ok(StatusCode::OK)
}
