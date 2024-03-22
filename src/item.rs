use serde::Serialize;
use uuid::fmt::Hyphenated;

use crate::progress::Progress;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub total: u64,
    pub size: u64,
    pub canceled: bool,
}

impl Item {
    pub fn from_progress(id: Hyphenated, pg: &Progress) -> Self {
        Self {
            id: id.to_string(),
            name: pg.name().to_string(),
            total: pg.total(),
            size: pg.size(),
            canceled: pg.is_canceled(),
        }
    }
}
