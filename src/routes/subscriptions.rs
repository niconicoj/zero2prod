use axum::{extract, Extension, Json};
use chrono::Utc;
use hyper::StatusCode;
use sea_orm::{
    prelude::{DateTimeWithTimeZone, Uuid},
    *,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::entities::{subscription::Model, *};

use crate::routes::error::Error;

use super::error::handle_db_error;

#[derive(Deserialize, Debug)]
pub struct SubscriptionRequest {
    name: String,
    email: String,
}

#[derive(Serialize)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub subscribed_at: DateTimeWithTimeZone,
}

#[instrument(name = "Adding a new subscriber")]
pub async fn subscribe(
    extract::Json(payload): extract::Json<SubscriptionRequest>,
    Extension(ref conn): Extension<DatabaseConnection>,
) -> Result<(StatusCode, Json<SubscriptionResponse>), (StatusCode, Json<Error>)> {
    let res = insert_subscriber(&payload, &conn)
        .await
        .map_err(|err| handle_db_error(err))?;

    Ok((
        StatusCode::CREATED,
        Json(SubscriptionResponse {
            id: res.id,
            name: res.name,
            email: res.email,
            subscribed_at: res.subscribed_at,
        }),
    ))
}

#[instrument(name = "Saving new subscriber in the database")]
async fn insert_subscriber(
    payload: &SubscriptionRequest,
    conn: &DatabaseConnection,
) -> Result<Model, DbErr> {
    subscription::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        name: ActiveValue::Set(payload.name.clone()),
        email: ActiveValue::Set(payload.email.clone()),
        subscribed_at: ActiveValue::Set(DateTimeWithTimeZone::from(Utc::now())),
    }
    .insert(conn)
    .await
}
