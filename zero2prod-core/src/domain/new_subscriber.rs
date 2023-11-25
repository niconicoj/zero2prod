use email_address::EmailAddress;
use serde::Deserialize;

use super::SubscriberName;

#[derive(Deserialize)]
pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: EmailAddress,
}
