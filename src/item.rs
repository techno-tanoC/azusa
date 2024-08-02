use serde::Serialize;
use uuid::fmt::Hyphenated;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    pub id: Hyphenated,
    pub url: String,
    pub name: String,
    pub ext: String,
    pub total: u64,
    pub size: u64,
    pub is_canceled: bool,
}
