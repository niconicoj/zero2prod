use async_trait::async_trait;

use crate::{domain::NewSubscriber, error::CoreResult};

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait SubscriptionRepository {
    async fn create(&self, new_subscriber: &NewSubscriber) -> CoreResult<()>;
}
