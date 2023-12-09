use std::sync::Arc;

use async_trait::async_trait;
use email_address::EmailAddress;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use zero2prod_core::{domain::Document, error::CoreResult, service::email_service::EmailService};

use crate::template::TemplateEngine;

pub struct EmailServiceImpl {
    sender: EmailAddress,
    http_client: Client,
    base_url: String,
    auth_token: SecretString,
    template_engine: Arc<TemplateEngine>,
}

impl EmailServiceImpl {
    pub fn from_config(
        config: &crate::configuration::EmailClientConfig,
        template_engine: Arc<TemplateEngine>,
    ) -> Self {
        Self {
            sender: config.sender_email.clone(),
            http_client: Client::builder()
                .timeout(std::time::Duration::from_millis(config.timeout))
                .build()
                .unwrap(),
            base_url: config.base_url.clone(),
            auth_token: config.auth_token.clone(),
            template_engine,
        }
    }
}

#[async_trait]
impl EmailService for EmailServiceImpl {
    async fn send_email(&self, recipient: &str, document: Document) -> CoreResult<()> {
        let url = format!("{}/email", self.base_url);

        let html = self.template_engine.render(&document);

        let json = json!({
            "from": self.sender.as_ref(),
            "to": recipient,
            "subject": document.title,
            "HtmlBody": html
        });

        self.http_client
            .post(&url)
            .json(&json)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
