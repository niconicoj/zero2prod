use email_address::EmailAddress;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct EmailClient {
    sender: EmailAddress,
    http_client: Client,
    base_url: String,
    auth_token: SecretString,
}

impl EmailClient {
    pub fn from_config(config: &crate::configuration::EmailClientConfig) -> Self {
        Self {
            http_client: Client::builder()
                .timeout(std::time::Duration::from_millis(config.timeout))
                .build()
                .unwrap(),
            sender: config.sender_email.clone(),
            base_url: config.base_url.clone(),
            auth_token: config.auth_token.clone(),
        }
    }

    pub async fn send_email(
        &self,
        recipient: &EmailAddress,
        subject: &str,
        html: &str,
        text: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);

        let json = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html,
            text_body: text,
        };

        let _ = self
            .http_client
            .post(&url)
            .json(&json)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::configuration::EmailClientConfig;

    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::{header, header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct SendEmailRequestMatcher;

    impl wiremock::Match for SendEmailRequestMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            match serde_json::from_slice::<serde_json::Value>(&request.body) {
                Ok(body) => {
                    body.get("From").is_some()
                        && body.get("To").is_some()
                        && body.get("Subject").is_some()
                        && body.get("HtmlBody").is_some()
                        && body.get("TextBody").is_some()
                }
                Err(_) => false,
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email_address() -> EmailAddress {
        EmailAddress::from_str(&SafeEmail().fake::<String>()).unwrap()
    }

    fn email_client(base_url: String, timeout: u64) -> EmailClient {
        let config = EmailClientConfig {
            base_url,
            sender_email: email_address(),
            auth_token: Secret::new(Faker.fake()),
            timeout,
        };
        EmailClient::from_config(&config)
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri(), 1000);

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailRequestMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = email_address();
        let subject: String = subject();
        let content: String = content();

        let result = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn send_email_fails_on_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri(), 1000);

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailRequestMatcher)
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = email_address();
        let subject: String = subject();
        let content: String = content();

        let result = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn send_email_fails_on_timeout() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri(), 100);

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailRequestMatcher)
            .respond_with(
                ResponseTemplate::new(200).set_delay(std::time::Duration::from_millis(300)),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = email_address();
        let subject: String = subject();
        let content: String = content();

        let result = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;

        assert!(result.is_err());
    }
}
