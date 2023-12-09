use std::fmt::Display;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    EmailAlreadyExists,
    InvalidDomain(String),
    Unexpected(String),
}

impl Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::EmailAlreadyExists => write!(f, "Email already exists"),
            CoreError::InvalidDomain(msg) => write!(f, "Invalid domain: {}", msg),
            CoreError::Unexpected(msg) => write!(f, "Unexpected error: {}", msg),
        }
    }
}

pub type CoreResult<T> = Result<T, CoreError>;

impl<T> From<T> for CoreError
where
    T: std::error::Error,
{
    fn from(value: T) -> Self {
        CoreError::Unexpected(value.to_string())
    }
}
