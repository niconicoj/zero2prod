use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(greet))
        .route("/:name", get(greet_name));

    let bind_address = "0.0.0.0:3000";
    info!("listening on {bind_address}");
    axum::Server::bind(&bind_address.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn greet() -> impl IntoResponse {
    "Hello World"
}

async fn greet_name(Path(name): Path<String>) -> impl IntoResponse {
    format!("Hello {name}")
}
