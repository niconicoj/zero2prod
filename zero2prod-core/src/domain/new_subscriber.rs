use email_address::EmailAddress;
use serde::{Deserialize, Serialize};

use super::SubscriberName;

#[derive(Serialize, Deserialize)]
pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: EmailAddress,
}
