use async_trait::async_trait;

#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::{domain::Document, error::CoreResult};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait EmailService {
    async fn send_email(&self, recipient: &str, document: Document) -> CoreResult<()>;
}
