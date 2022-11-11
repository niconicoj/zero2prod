use std::time::Duration;

use axum::{http::HeaderValue, routing::IntoMakeService, Extension, Router, Server};
use configuration::Settings;
use db::Db;
use hyper::{header::HeaderName, server::conn::AddrIncoming, Request, Response};
use routes::router;
use sea_orm::{prelude::Uuid, DatabaseConnection, DbErr};
use tower::ServiceBuilder;
use tower_http::{
    classify::ServerErrorsFailureClass,
    request_id::{MakeRequestId, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{error, info, info_span, Span};

mod routes;
mod startup;

pub mod configuration;
pub mod db;
pub mod entities;
pub mod migrations;
pub mod telemetry;

#[derive(Clone, Default)]
struct UuidMakeRequestId;

impl MakeRequestId for UuidMakeRequestId {
    fn make_request_id<B>(
        &mut self,
        _request: &Request<B>,
    ) -> Option<tower_http::request_id::RequestId> {
        let request_id = Uuid::new_v4().to_string().parse().unwrap();
        Some(RequestId::new(request_id))
    }
}

pub async fn run(
    settings: &Settings,
) -> Result<
    (
        Server<AddrIncoming, IntoMakeService<Router>>,
        DatabaseConnection,
    ),
    DbErr,
> {
    let conn = Db::init_db_connection(&settings.database);

    let x_request_id = HeaderName::from_static("x-request-id");

    let addr = format!("{}:{}", settings.app.host, settings.app.port);
    let listener =
        std::net::TcpListener::bind(&addr).expect(&format!("failed to bind to {}", addr));

    Ok((
        axum::Server::from_tcp(listener).unwrap().serve(
            router()
                .layer(
                    ServiceBuilder::new()
                        .layer(Extension(conn.clone()))
                        .layer(SetRequestIdLayer::new(
                            x_request_id.clone(),
                            UuidMakeRequestId::default(),
                        ))
                        .layer(
                            TraceLayer::new_for_http()
                                .make_span_with(|_request: &Request<_>| {
                                    info_span!(
                                        "request",
                                        request_id = tracing::field::Empty,
                                        version = tracing::field::Empty,
                                        method = tracing::field::Empty,
                                        uri = tracing::field::Empty,
                                        status_code = tracing::field::Empty
                                    )
                                })
                                .on_request(|req: &Request<_>, span: &Span| {
                                    let default_id = HeaderValue::from_static("");
                                    let request_id = req
                                        .headers()
                                        .get("x-request-id")
                                        .unwrap_or(&default_id)
                                        .to_str()
                                        .expect("non ascii uuid");
                                    span.record("request_id", &tracing::field::display(request_id));
                                    span.record("version", &tracing::field::debug(req.version()));
                                    span.record("method", &tracing::field::display(req.method()));
                                    span.record("uri", &tracing::field::display(req.uri()));
                                })
                                .on_response(|res: &Response<_>, lat: Duration, span: &Span| {
                                    span.record(
                                        "status_code",
                                        &tracing::field::display(res.status().as_u16()),
                                    );
                                    if lat.as_millis() > 0 {
                                        info!("request took {}ms", lat.as_millis());
                                    } else {
                                        info!("request took {}µs", lat.as_micros());
                                    }
                                })
                                .on_failure(
                                    |error: ServerErrorsFailureClass,
                                     _lat: Duration,
                                     _span: &Span| {
                                        error!("{}", error);
                                    },
                                ),
                        ),
                )
                .into_make_service(),
        ),
        conn.clone(),
    ))
}
