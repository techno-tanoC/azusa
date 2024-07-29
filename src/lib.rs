use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
pub use item::Item;
use persist::Persist;
use progress::Progress;
use progress_map::ProgressMap;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

mod item;
mod persist;
mod progress;
mod progress_map;

#[derive(Debug)]
pub struct Engine {
    client: reqwest::Client,
    map: ProgressMap,
    persist: Persist,
}

impl Engine {
    pub fn new(dest: impl Into<PathBuf>) -> Self {
        let client = reqwest::Client::new();
        let map = ProgressMap::default();
        let persist = Persist::new(dest.into());
        Self {
            client,
            map,
            persist,
        }
    }

    pub async fn download(
        &self,
        url: impl Into<String>,
        title: impl Into<String>,
        ext: impl Into<String>,
        threshold: Option<u64>,
    ) -> Result<bool> {
        let url = url.into();
        let title = title.into();
        let ext = ext.into();

        let id = Uuid::now_v7();
        let pg = Arc::new(Progress::new(url.clone(), title.clone(), ext.clone()));

        self.map.insert(id, pg.clone()).await;
        let result = self.do_download(&url, &title, &ext, pg, threshold).await;
        self.map.remove(id).await;

        result
    }

    async fn do_download(
        &self,
        url: &str,
        title: &str,
        ext: &str,
        pg: Arc<Progress>,
        threshold: Option<u64>,
    ) -> Result<bool> {
        let mut temp = async_tempfile::TempFile::new().await?;
        let mut response = self.client.get(url).send().await?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("{} {}", status.as_u16(), status.as_str());
        }

        // Set Content-Length
        if let Some(total) = response.content_length() {
            pg.set_total(total);
        }

        // Download
        while let Some(chunk) = response.chunk().await? {
            if pg.is_canceled() {
                return Ok(false);
            }
            temp.write_all(&chunk).await?;
            pg.progress(chunk.len() as u64);
        }

        // Check file size
        let len = temp.metadata().await?.len();
        if let Some(size) = threshold {
            if size > len {
                anyhow::bail!("{} < {}", len, size);
            }
        }

        // Persist
        self.persist.persist(title, ext, &mut temp).await?;

        Ok(true)
    }

    pub async fn index(&self) -> Vec<Item> {
        self.map.to_items().await
    }

    pub async fn abort(&self, id: Uuid) {
        self.map.abort(id).await;
    }
}
