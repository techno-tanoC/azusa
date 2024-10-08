use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use uuid::Uuid;

use crate::item::Item;

#[derive(Debug)]
pub struct Progress {
    url: String,
    name: String,
    ext: String,
    total: AtomicU64,
    size: AtomicU64,
    is_canceled: AtomicBool,
}

impl Progress {
    pub fn new(url: impl Into<String>, name: impl Into<String>, ext: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            name: name.into(),
            ext: ext.into(),
            total: AtomicU64::new(0),
            size: AtomicU64::new(0),
            is_canceled: AtomicBool::new(false),
        }
    }

    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    pub fn size(&self) -> u64 {
        self.size.load(Ordering::Relaxed)
    }

    pub fn is_canceled(&self) -> bool {
        self.is_canceled.load(Ordering::Relaxed)
    }

    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
    }

    pub fn progress(&self, delta: u64) {
        self.size.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn cancel(&self) {
        self.is_canceled.store(true, Ordering::Relaxed);
    }

    pub fn to_item(&self, id: Uuid) -> Item {
        Item {
            id: id.hyphenated(),
            url: self.url.to_string(),
            name: self.name.to_string(),
            ext: self.ext.to_string(),
            total: self.total(),
            size: self.size(),
            is_canceled: self.is_canceled(),
        }
    }
}
