use axum::Json;
use hyper::StatusCode;
use sea_orm::DbErr;
use serde::Serialize;
use tracing::error;

#[derive(Serialize)]
pub struct Error {
    message: String,
}

pub fn error(message: String) -> Json<Error> {
    return Json(Error { message });
}

pub fn handle_db_error(err: DbErr) -> (StatusCode, Json<Error>) {
    error!("{}", err);
    match err {
        DbErr::Query(_) => (StatusCode::BAD_REQUEST, error("bad request".to_string())),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            error("the request could not be handled due to an internal error".to_string()),
        ),
    }
}
