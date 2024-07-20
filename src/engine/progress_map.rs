use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::Mutex;
use uuid::Uuid;

use super::{item::Item, progress::Progress};

#[derive(Debug, Clone)]
pub struct ProgressMap(Arc<Mutex<BTreeMap<Uuid, Arc<Progress>>>>);

impl Default for ProgressMap {
    fn default() -> Self {
        let map = BTreeMap::new();
        Self(Arc::new(Mutex::new(map)))
    }
}

impl ProgressMap {
    pub async fn add(&self, id: Uuid, progress: Arc<Progress>) {
        let mut map = self.0.lock().await;
        map.insert(id, progress);
    }

    pub async fn remove(&self, id: Uuid) {
        let mut map = self.0.lock().await;
        map.remove(&id);
    }

    pub async fn cancel(&self, id: Uuid) {
        let map = self.0.lock().await;
        if let Some(progress) = map.get(&id) {
            progress.cancel();
        } else {
            println!("progress not found on cancel: {}", id.as_hyphenated());
        }
    }

    pub async fn to_items(&self) -> Vec<Item> {
        let map = self.0.lock().await;
        let mut items = vec![];
        for (key, pg) in map.iter() {
            let item = pg.to_item(*key);
            items.push(item);
        }
        items
    }
}
