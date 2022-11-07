use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, prelude::*, EnvFilter, Registry};

use crate::configuration::Settings;

pub fn get_subscriber<Sink>(config: &Settings, sink: Sink) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::new(&config.env_filter);
    let formatting_layer = BunyanFormattingLayer::new(config.app.name.clone(), sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(sub: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("failed to init logger");
    set_global_default(sub).expect("failed to setup subscriber");
}
