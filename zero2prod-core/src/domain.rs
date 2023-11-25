use std::fmt;

use email_address::EmailAddress;
use serde::{de::Visitor, Deserialize};

static FORBIDDEN_CHARS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub name: SubscriberName,
    pub email: EmailAddress,
}

pub struct SubscriberName(String);

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SubscriberName {
    fn deserialize<D>(deserializer: D) -> Result<SubscriberName, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(SubscriberNameVisitor)
    }
}

pub struct SubscriberNameVisitor;

impl<'de> Visitor<'de> for SubscriberNameVisitor {
    type Value = SubscriberName;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid subscriber name")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value.is_empty() || value.len() > 256 {
            return Err(serde::de::Error::invalid_length(value.len(), &self));
        }
        for c in value.chars() {
            if FORBIDDEN_CHARS.contains(&c) {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Char(c),
                    &self,
                ));
            }
        }
        Ok(SubscriberName(value.to_owned()))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn deserialize_valid_subscriber_name() {
        let input = r#""Giovanni" "#;
        let expected = SubscriberName("Giovanni".to_owned());
        let actual: SubscriberName = serde_json::from_str(input).unwrap();
        assert_eq!(expected.0, actual.0);
    }

    #[test]
    fn deserialize_invalid_subscriber_name() {
        let input = r#""Giovanni\\ ""#;
        let actual: Result<SubscriberName, _> = serde_json::from_str(input);
        assert!(actual.is_err());
    }

    #[test]
    fn deserialize_empty_subscriber_name() {
        let input = r#""""#;
        let actual: Result<SubscriberName, _> = serde_json::from_str(input);
        assert!(actual.is_err());
    }

    #[test]
    fn deserialize_valid_email() {
        let input = r#""giovanni@localhost""#;
        let actual: EmailAddress = serde_json::from_str(input).unwrap();
        assert_eq!(actual.as_str(), "giovanni@localhost");
    }
}
