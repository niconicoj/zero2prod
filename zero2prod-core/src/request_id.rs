use std::{
    error::Error,
    fmt::Display,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{async_trait, extract::FromRequestParts};
use hyper::{http::request::Parts, service::Service, Request, StatusCode};
use tower_layer::Layer;
use uuid::Uuid;

use crate::error::internal_error;

#[derive(Clone, Debug)]
pub struct RequestId(pub Arc<Uuid>);

impl RequestId {
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
impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let request_id = parts
            .extensions
            .get::<RequestId>()
            .ok_or(RequestIdError::NotPresent)
            .map_err(internal_error)?;
        Ok(request_id.clone())
    }
}

impl Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct RequestIdService<S> {
    inner: S,
}

impl<B, S> Service<Request<B>> for RequestIdService<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let request_id = RequestId::generate();
        req.extensions_mut().insert(request_id);
        self.inner.call(req)
    }
}

#[derive(Clone, Debug)]
pub struct RequestIdLayer;

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}
