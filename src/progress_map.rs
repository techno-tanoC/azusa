use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{item::Item, progress::Progress};

#[derive(Debug, Default)]
pub struct ProgressMap {
    map: Arc<Mutex<BTreeMap<Uuid, Arc<Progress>>>>,
}

impl ProgressMap {
    pub async fn insert(&self, key: Uuid, pg: Arc<Progress>) {
        let mut map = self.map.lock().await;
        map.insert(key, pg);
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
        map.iter().map(|(id, pg)| pg.to_item(*id)).collect()
    }
}
