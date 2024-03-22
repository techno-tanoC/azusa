use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug)]
pub struct Progress {
    name: String,
    total: AtomicU64,
    size: AtomicU64,
    canceled: AtomicBool,
}

impl Progress {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            total: AtomicU64::new(0),
            size: AtomicU64::new(0),
            canceled: AtomicBool::new(false),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    pub fn size(&self) -> u64 {
        self.size.load(Ordering::Relaxed)
    }

    pub fn is_canceled(&self) -> bool {
        self.canceled.load(Ordering::Relaxed)
    }

    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
    }

    pub fn progress(&self, delta: u64) {
        self.size.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn cancel(&self) {
        self.canceled.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_progress() {
        let name = "example";
        let pg = Progress::new(name);

        assert_eq!(pg.name(), name);
        assert_eq!(pg.total(), 0);
        assert_eq!(pg.size(), 0);
        assert!(!pg.is_canceled());

        pg.set_total(1000);
        pg.progress(100);
        pg.cancel();

        assert_eq!(pg.name(), name);
        assert_eq!(pg.total(), 1000);
        assert_eq!(pg.size(), 100);
        assert!(pg.is_canceled());

        pg.progress(300);

        assert_eq!(pg.name(), name);
        assert_eq!(pg.total(), 1000);
        assert_eq!(pg.size(), 400);
        assert!(pg.is_canceled());
    }
}
