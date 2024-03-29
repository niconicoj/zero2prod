use std::{
    fmt::{self, Display, Formatter},
    sync::Arc,
};

use crate::{configuration::Configuration, service::EmailServiceImpl, template::TemplateEngine};
use axum::{
    routing::{get, post, IntoMakeService},
    serve::Serve,
    Extension, Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

use tokio::net::TcpListener;

use crate::{
    configuration::WithDb,
    handlers::{health_check, subscribe},
    layer::TraceIdLayer,
};

#[derive(Default, Clone)]
pub struct Address {
    pub host: String,
    pub port: u16,
}

pub type Server = Serve<IntoMakeService<Router>, Router>;

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "http://{}:{}", self.host, self.port)
    }
}

pub async fn start(configuration: &Configuration) -> (Server, Address, PgPool) {
    let address = format!("{}:{}", configuration.app.host, configuration.app.port);
    let listener = TcpListener::bind(address)
        .await
        .expect("Failed to bind listener");

    info!("Setting up database connection pool");
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(configuration.db.timeout))
        .connect_lazy_with(configuration.db.connection_options(WithDb::Yes));

    let template_engine = Arc::new(TemplateEngine::init());

    let email_client = Arc::new(EmailServiceImpl::from_config(
        &configuration.email_client,
        template_engine.clone(),
    ));

    let subscription_repository = Arc::new(crate::repository::SubscriptionRepositoryImpl::new(
        pool.clone(),
    ));

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(pool.clone())
        .layer(Extension(email_client))
        .layer(Extension(subscription_repository))
        .layer(Extension(Arc::new(configuration.clone())))
        .layer(TraceIdLayer);

    let app_address = Address {
        host: listener.local_addr().unwrap().ip().to_string(),
        port: listener.local_addr().unwrap().port(),
    };

    let addr = listener
        .local_addr()
        .expect("Failed to get listener address");

    info!("listening on {}", addr);
    (
        axum::serve(listener, app.into_make_service()),
        app_address,
        pool,
    )
}
