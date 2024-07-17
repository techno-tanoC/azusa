use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug)]
pub struct Progress {
    url: String,
    title: String,
    ext: String,
    total: AtomicU64,
    current: AtomicU64,
    is_canceled: AtomicBool,
}

impl Progress {
    pub fn new(url: impl Into<String>, title: impl Into<String>, ext: impl Into<String>) -> Self {
        let url = url.into();
        let title = title.into();
        let ext = ext.into();
        let total = AtomicU64::new(0);
        let current = AtomicU64::new(0);
        let is_canceled = AtomicBool::new(false);
        Self {
            url,
            title,
            ext,
            total,
            current,
            is_canceled,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn ext(&self) -> &str {
        &self.ext
    }

    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    pub fn current(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }

    pub fn is_canceled(&self) -> bool {
        self.is_canceled.load(Ordering::Relaxed)
    }

    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
    }

    pub fn progress(&self, delta: u64) {
        self.current.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn cancel(&self) {
        self.is_canceled.store(true, Ordering::Relaxed);
    }
}
