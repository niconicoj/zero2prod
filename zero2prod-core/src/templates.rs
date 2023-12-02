use std::sync::Arc;

use axum::extract::FromRef;
use handlebars::Handlebars;
use serde::Serialize;

pub static CONFIRMATION_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/email/confirmation.html"
));

pub static CONFIRMATION_TXT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/email/confirmation.txt"
));

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Template<'a> {
    ConfirmationHtml { link: &'a str },
    ConfirmationTxt { link: &'a str },
}

impl<'a> Template<'a> {
    pub fn key(&self) -> &'static str {
        match self {
            Self::ConfirmationHtml { .. } => "confirmation.html",
            Self::ConfirmationTxt { .. } => "confirmation.txt",
        }
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
        engine
            .register_template_string("confirmation.txt", CONFIRMATION_TXT)
            .expect("Failed to register confirmation.txt template");
        Self {
            engine: Arc::new(engine),
        }
    }

    pub fn render(&self, template: Template) -> String {
        self.engine
            .render(template.key(), &template)
            .expect("Failed to render template")
    }
}
