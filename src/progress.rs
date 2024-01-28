use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug)]
pub struct Progress {
    pub name: String,
    pub total: AtomicU64,
    pub size: AtomicU64,
    pub canceled: AtomicBool,
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
        let name = "example".to_string();
        let pg = Progress::new(&name);

        assert_eq!(pg.name, name);
        assert_eq!(pg.total.load(Ordering::SeqCst), 0);
        assert_eq!(pg.size.load(Ordering::SeqCst), 0);
        assert!(!pg.canceled.load(Ordering::SeqCst));

        pg.set_total(1000);
        pg.progress(100);
        pg.cancel();

        assert_eq!(pg.name, name);
        assert_eq!(pg.total.load(Ordering::SeqCst), 1000);
        assert_eq!(pg.size.load(Ordering::SeqCst), 100);
        assert!(pg.canceled.load(Ordering::SeqCst));
    }
}
