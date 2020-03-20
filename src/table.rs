use indexmap::IndexMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::progress::Progress;
use super::item::Item;

pub struct Table<K, V>(Arc<Mutex<IndexMap<K, V>>>);

impl<K: Hash + Eq, V> Table<K, V> {
    pub fn new() -> Self {
        Table(Arc::new(Mutex::new(IndexMap::new())))
    }

    pub async fn add(&self, id: K, v: V) {
        self.0.lock().await.insert(id, v);
    }

    pub async fn delete(&self, id: impl AsRef<K>) {
        self.0.lock().await.shift_remove(id.as_ref());
    }
}

impl<T> Table<String, Progress<T>> {
    pub async fn cancel(&self, id: impl AsRef<str>) {
        if let Some(pg) = self.0.lock().await.get_mut(id.as_ref()) {
            pg.cancel().await;
        }
    }

    pub async fn to_vec(&self) -> Vec<Item> {
        let mut vec = vec![];
        for (k, v) in self.0.lock().await.iter() {
            vec.push(v.to_item(k.clone()).await);
        }
        vec
    }
}
