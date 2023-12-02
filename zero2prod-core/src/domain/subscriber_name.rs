use std::fmt;

use serde::{de::Visitor, Deserialize, Serialize};

static FORBIDDEN_CHARS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

#[derive(Serialize)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() || trimmed.len() > 256 {
            return Err("Invalid length".into());
        }
        for c in trimmed.chars() {
            if FORBIDDEN_CHARS.contains(&c) {
                return Err("Invalid character".into());
            }
        }
        Ok(SubscriberName(trimmed))
    }
}

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

struct SubscriberNameVisitor;

impl<'de> Visitor<'de> for SubscriberNameVisitor {
    type Value = SubscriberName;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid subscriber name")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match SubscriberName::parse(value.to_owned()) {
            Ok(name) => Ok(name),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
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
}
