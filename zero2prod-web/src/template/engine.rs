use std::sync::Arc;

use axum::extract::FromRef;
use handlebars::Handlebars;
use zero2prod_core::domain::Document;

pub static CONFIRMATION_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/template/resources/email/confirmation.html"
));

fn key(document: &Document) -> &'static str {
    match document.kind {
        zero2prod_core::domain::DocumentKind::Confirmation { .. } => "confirmation.html",
    }
}

#[derive(Clone, FromRef)]
pub struct TemplateEngine {
    engine: Arc<Handlebars<'static>>,
}

impl TemplateEngine {
    pub fn init() -> Self {
        let mut engine = Handlebars::new();
        engine
            .register_template_string("confirmation.html", CONFIRMATION_HTML)
            .expect("Failed to register confirmation.html template");
        Self {
            engine: Arc::new(engine),
        }
    }

    pub fn render(&self, document: &Document) -> String {
        self.engine
            .render(key(document), &document.kind)
            .expect("Failed to render template")
    }
}
