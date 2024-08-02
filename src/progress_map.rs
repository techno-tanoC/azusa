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

#[cfg(test)]
mod tests {
    use super::Progress;
    use super::*;

    const URL: &str = "url";
    const NAME: &str = "name";
    const EXT: &str = "jpg";

    #[tokio::test]
    async fn test_insert_and_remove() {
        let id = Uuid::now_v7();
        let pg = Arc::new(Progress::new(URL, NAME, EXT));
        let map = ProgressMap::default();

        assert_eq!(map.to_items().await, vec![]);

        map.insert(id, pg).await;
        assert_eq!(
            map.to_items().await,
            vec![Item {
                id: id.hyphenated(),
                url: URL.to_string(),
                name: NAME.to_string(),
                ext: EXT.to_string(),
                total: 0,
                size: 0,
                is_canceled: false,
            }]
        );

        map.remove(id).await;
        assert_eq!(map.to_items().await, vec![]);
    }

    #[tokio::test]
    async fn test_abort() {
        let id = Uuid::now_v7();
        let pg = Arc::new(Progress::new(URL, NAME, EXT));
        let map = ProgressMap::default();

        map.insert(id, pg.clone()).await;
        map.abort(id).await;
        assert_eq!(map.to_items().await, vec![]);
        assert!(pg.is_canceled());
    }

    #[tokio::test]
    async fn test_to_items() {
        let map = ProgressMap::default();
        for _ in 0..10 {
            let id = Uuid::now_v7();
            let pg = Arc::new(Progress::new(URL, NAME, EXT));
            map.insert(id, pg).await;
        }

        assert_eq!(map.to_items().await.len(), 10);
    }
}
