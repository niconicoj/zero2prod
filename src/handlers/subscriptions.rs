use axum::{extract::rejection::FormRejection, response::IntoResponse, Form};
use hyper::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SubscribeRequest {
    name: String,
    email: String,
}

pub async fn subscribe(form: Result<Form<SubscribeRequest>, FormRejection>) -> impl IntoResponse {
    match form {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}
