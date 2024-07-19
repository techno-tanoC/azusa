use serde::Serialize;
use uuid::fmt::Hyphenated;

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub id: Hyphenated,
    pub url: String,
    pub title: String,
    pub ext: String,
    pub total: u64,
    pub current: u64,
    pub is_canceled: bool,
}
