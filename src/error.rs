use axum::extract::rejection::FormRejection;
use hyper::StatusCode;

pub fn internal_error<E: std::error::Error + Send + Sync + 'static>(
    err: E,
) -> (StatusCode, String) {
    tracing::error!("Internal server error: {}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".to_string(),
    )
}

pub fn db_error(err: sqlx::Error) -> (StatusCode, String) {
    match err {
        sqlx::Error::Database(err) if err.is_unique_violation() => {
            (StatusCode::CONFLICT, "Email already exists".to_string())
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
