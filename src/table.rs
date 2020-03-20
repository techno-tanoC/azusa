use indexmap::IndexMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::progress::Progress;
use super::item::Item;

pub struct Table<K, V>(Arc<Mutex<IndexMap<K, V>>>);

impl<V> Table<String, V> {
    pub fn new() -> Self {
        Table(Arc::new(Mutex::new(IndexMap::new())))
    }

    pub async fn add(&self, id: impl ToString, value: V) {
        self.0.lock().await.insert(id.to_string(), value);
    }

    pub async fn delete(&self, id: impl AsRef<str>) {
        self.0.lock().await.shift_remove(id.as_ref());
    }
}

impl<T> Table<String, Progress<T>> {
    pub async fn cancel(&self, id: impl AsRef<str>) {
        if let Some(pg) = self.0.lock().await.get_mut(id.as_ref()) {
            pg.cancel().await;
        }
    }

    pub async fn to_items(&self) -> Vec<Item> {
        let mut vec = vec![];
        for (k, v) in self.0.lock().await.iter() {
            vec.push(v.to_item(k.clone()).await);
        }
        vec
    }
}

impl<K, V> std::clone::Clone for Table<K, V> {
    fn clone(&self) -> Self {
        Table(self.0.clone())
    }
}
