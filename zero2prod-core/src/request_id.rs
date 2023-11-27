use std::{error::Error, fmt::Display, sync::Arc};

use axum::{async_trait, body::Body, extract::FromRequestParts};
use http::{Request, StatusCode};
use hyper::{http::request::Parts, service::Service};
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

impl Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct TraceIdService<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for TraceIdService<S>
where
    S: Service<Request<Body>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Instrumented<S::Future>;

    fn call(&self, mut req: Request<Body>) -> Self::Future {
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
