use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::Mutex;
use uuid::Uuid;

use super::{item::Item, progress::Progress};

type RawMap = Arc<Mutex<BTreeMap<Uuid, Arc<Progress>>>>;

#[derive(Debug, Clone)]
pub struct ProgressMap(RawMap);

impl Default for ProgressMap {
    fn default() -> Self {
        let map = BTreeMap::new();
        Self(Arc::new(Mutex::new(map)))
    }
}

impl ProgressMap {
    pub async fn add(&self, id: Uuid, progress: Arc<Progress>) -> EntryGuard {
        let mut map = self.0.lock().await;
        map.insert(id, progress);
        EntryGuard {
            map: self.0.clone(),
            id,
        }
    }

    pub async fn remove(&self, id: Uuid) {
        let mut map = self.0.lock().await;
        map.remove(&id);
    }

    pub async fn cancel(&self, id: Uuid) {
        let map = self.0.lock().await;
        if let Some(progress) = map.get(&id) {
            progress.cancel();
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

pub struct EntryGuard {
    map: RawMap,
    id: Uuid,
}

impl Drop for EntryGuard {
    fn drop(&mut self) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut map = self.map.lock().await;
                map.remove(&self.id)
            });
        });
    }
}
