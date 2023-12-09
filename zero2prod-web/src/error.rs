use axum::extract::rejection::FormRejection;
use hyper::StatusCode;
use zero2prod_core::error::CoreError;

pub fn core_error(err: CoreError) -> (StatusCode, String) {
    match err {
        CoreError::EmailAlreadyExists => {
            (StatusCode::BAD_REQUEST, "email already exists".to_string())
        }
        CoreError::InvalidDomain(message) => (
            StatusCode::BAD_REQUEST,
            format!("invalid data: {}", message),
        ),
        CoreError::Unexpected(message) => {
            tracing::error!("Internal server error: {}", message);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            )
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        ),
    }
}

pub fn form_rejection(err: FormRejection) -> (StatusCode, String) {
    tracing::info!("Bad request: {}", err.body_text());
    (StatusCode::BAD_REQUEST, err.body_text())
}
