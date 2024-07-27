use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{item::Item, progress::Progress};

#[derive(Debug, Default)]
pub struct ProgressMap {
    map: Arc<Mutex<BTreeMap<Uuid, Arc<Progress>>>>,
}

impl ProgressMap {
    pub async fn insert(&self, key: Uuid, pg: Arc<Progress>) -> EntryGuard {
        let mut map = self.map.lock().await;
        map.insert(key, pg);
        EntryGuard {
            key,
            map: self.map.clone(),
        }
    }

    pub async fn remove(&self, key: Uuid) {
        let mut map = self.map.lock().await;
        map.remove(&key);
    }

    pub async fn abort(&self, key: Uuid) {
        let mut map = self.map.lock().await;
        if let Some(pg) = map.remove(&key) {
            pg.cancel();
        }
    }

    pub async fn to_items(&self) -> Vec<Item> {
        let map = self.map.lock().await;
        map.values().map(|pg| pg.to_item()).collect()
    }
}

#[derive(Debug)]
pub struct EntryGuard {
    key: Uuid,
    map: Arc<Mutex<BTreeMap<Uuid, Arc<Progress>>>>,
}

impl Drop for EntryGuard {
    fn drop(&mut self) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut map = self.map.lock().await;
                map.remove(&self.key);
            })
        })
    }
}
