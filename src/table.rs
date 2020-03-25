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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::item::Item;
    use crate::progress::Progress;

    #[tokio::test]
    async fn add_test() {
        let table = Table::new();
        table.add("1", true).await;
        table.add("2", false).await;
        assert_eq!(table.0.lock().await.iter().collect::<Vec<_>>(), vec![(&"1".to_string(), &true), (&"2".to_string(), &false)]);
    }

    #[tokio::test]
    async fn delete_test() {
        let table = Table::new();
        table.add("1", true).await;
        table.add("2", false).await;
        assert_eq!(table.0.lock().await.len(), 2);

        table.delete("1").await;
        assert_eq!(table.0.lock().await.iter().collect::<Vec<_>>(), vec![(&"2".to_string(), &false)]);
        table.delete("2").await;
        assert_eq!(table.0.lock().await.iter().collect::<Vec<_>>(), vec![]);
    }

    #[tokio::test]
    async fn cancel_test() {
        let table = Table::new();
        let pg = Progress::new("name", ());
        table.add("1", pg.clone()).await;

        assert!(!pg.to_item("1").await.canceled);
        table.cancel("1").await;
        assert!(pg.to_item("1").await.canceled);
    }

    #[tokio::test]
    async fn to_items_test() {
        let table = Table::new();
        let pg1 = Progress::new("name", ());
        table.add("1", pg1.clone()).await;
        let pg2 = Progress::new("name", ());
        table.add("2", pg2.clone()).await;

        assert_eq!(
            table.to_items().await,
            vec![
                Item {
                    id: "1".to_string(),
                    name: "name".to_string(),
                    total: 0,
                    size: 0,
                    canceled: false
                },
                Item {
                    id: "2".to_string(),
                    name: "name".to_string(),
                    total: 0,
                    size: 0,
                    canceled: false
                },
            ]
        );
    }
}
