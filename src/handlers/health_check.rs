use axum::response::IntoResponse;
use hyper::StatusCode;

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
