use std::{
    fmt::Display,
    sync::Arc,
    task::{Context, Poll},
};

use axum::body::Body;
use http::Request;
use tower::Service;
use tower_layer::Layer;
use tracing::{instrument::Instrumented, trace_span, Instrument};
use uuid::Uuid;

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

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
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
