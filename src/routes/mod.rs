use axum::{
    routing::{get, post},
    Router,
};

mod error;
mod health_check;
mod subscriptions;

pub fn router() -> Router {
    Router::new()
        .route("/health_check", get(health_check::health_check))
        .route("/subscriptions", post(subscriptions::subscribe))
}
