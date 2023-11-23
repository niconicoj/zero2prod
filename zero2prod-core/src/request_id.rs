use std::{
    error::Error,
    fmt::Display,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{async_trait, extract::FromRequestParts};
use hyper::{http::request::Parts, service::Service, Request, StatusCode};
use tower_layer::Layer;
use tracing::{instrument::Instrumented, trace_span, Instrument};
use uuid::Uuid;

use crate::error::internal_error;

#[derive(Clone, Debug)]
pub struct TraceId(pub Arc<Uuid>);

impl TraceId {
    fn generate() -> Self {
        Self(Arc::new(Uuid::new_v4()))
    }
}

#[derive(Debug)]
enum RequestIdError {
    NotPresent,
}

impl Display for RequestIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestIdError::NotPresent => write!(f, "RequestId not present in request extensions"),
        }
    }
}

impl Error for RequestIdError {}

#[async_trait]
impl<S> FromRequestParts<S> for TraceId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let request_id = parts
            .extensions
            .get::<TraceId>()
            .ok_or(RequestIdError::NotPresent)
            .map_err(internal_error)?;
        Ok(request_id.clone())
    }
}

impl Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct TraceIdService<S> {
    inner: S,
}

impl<B, S> Service<Request<B>> for TraceIdService<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Instrumented<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let trace_id = TraceId::generate();
        let span = trace_span!("request", trace_id = trace_id.to_string());
        req.extensions_mut().insert(trace_id);
        self.inner.call(req).instrument(span)
    }
}

#[derive(Clone, Debug)]
pub struct TraceIdLayer;

impl<S> Layer<S> for TraceIdLayer {
    type Service = TraceIdService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        TraceIdService { inner }
    }
}
