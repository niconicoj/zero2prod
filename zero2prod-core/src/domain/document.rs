use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub title: String,
    pub kind: DocumentKind,
}

impl Document {
    pub fn new(title: String, kind: DocumentKind) -> Self {
        Self { title, kind }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum DocumentKind {
    Confirmation { confirmation_link: String },
}
