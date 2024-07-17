mod downloader;
mod item;
mod progress;
mod progress_map;

use std::sync::Arc;

use uuid::Uuid;

use downloader::Downloader;
use item::Item;
use progress::Progress;
use progress_map::ProgressMap;

#[derive(Debug, Default)]
pub struct Engine {
    map: ProgressMap,
}

impl Engine {
    pub async fn download(&self, url: String, title: String, ext: String) {
        let id = Uuid::now_v7();
        let pg = Arc::new(Progress::new(url, title, ext));

        self.map.add(id, pg.clone()).await;

        let client = reqwest::Client::new();
        let downloader = Downloader::new(client);
        let _ = downloader.start(pg).await;

        self.map.remove(id).await;
    }

    pub async fn to_items(&self) -> Vec<Item> {
        self.map.to_items().await
    }

    pub async fn cancel(&self, id: Uuid) {
        self.map.cancel(id).await;
    }
}
