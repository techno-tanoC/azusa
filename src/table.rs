use std::sync::{atomic::Ordering, Arc};

use indexmap::IndexMap;
use tokio::sync::Mutex;
use uuid::fmt::Hyphenated;

use crate::{progress::Progress, Item};

pub struct Table(Arc<Mutex<IndexMap<Hyphenated, Arc<Progress>>>>);

impl Table {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(IndexMap::new())))
    }

    pub async fn to_vec(&self) -> Vec<Item> {
        self.0
            .lock()
            .await
            .iter()
            .map(|(id, pg)| Item {
                id: id.to_string(),
                name: pg.name.clone(),
                total: pg.total.load(Ordering::Relaxed),
                size: pg.size.load(Ordering::Relaxed),
                canceled: pg.canceled.load(Ordering::Relaxed),
            })
            .collect()
    }

    pub async fn add(&self, pg: Arc<Progress>) -> Hyphenated {
        let id = uuid::Uuid::new_v4().hyphenated();
        self.0.lock().await.insert(id, pg);
        id
    }

    pub async fn delete(&self, id: Hyphenated) {
        self.0.lock().await.remove(&id);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_table() {
        let table = Table::new();

        let items = table.to_vec().await;
        assert_eq!(items, vec![]);

        let title = "test";
        let pg = Arc::new(Progress::new(title));
        let id = table.add(pg.clone()).await;

        let items = table.to_vec().await;
        assert_eq!(
            items,
            vec![Item {
                id: id.to_string(),
                name: title.to_string(),
                total: 0,
                size: 0,
                canceled: false,
            }]
        );

        pg.set_total(1000);
        pg.progress(100);
        pg.cancel();

        let items = table.to_vec().await;
        assert_eq!(
            items,
            vec![Item {
                id: id.to_string(),
                name: title.to_string(),
                total: 1000,
                size: 100,
                canceled: true,
            }]
        );

        table.delete(id).await;

        let items = table.to_vec().await;
        assert_eq!(items, vec![]);
    }
}
