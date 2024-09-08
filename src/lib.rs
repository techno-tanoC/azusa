use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
pub use item::Item;
use persist::Persist;
use progress::Progress;
use progress_map::ProgressMap;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

mod cert;
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
    pub async fn new(certs: impl Into<PathBuf>, dest: impl Into<PathBuf>) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .read_timeout(Duration::from_secs(30));
        for cert in cert::trusted_root_certificates(certs.into()).await? {
            builder = builder.add_root_certificate(cert);
        }
        let client = builder.build()?;

        let map = ProgressMap::default();
        let persist = Persist::new(dest.into());
        Ok(Self {
            client,
            map,
            persist,
        })
    }

    pub async fn download(
        &self,
        url: impl Into<String>,
        name: impl Into<String>,
        ext: impl Into<String>,
        threshold: Option<u64>,
    ) -> Result<()> {
        let url = url.into();
        let name = name.into();
        let ext = ext.into();

        let id = Uuid::now_v7();
        let pg = Arc::new(Progress::new(url.clone(), name.clone(), ext.clone()));

        self.map.insert(id, pg.clone()).await;
        let result = self.do_download(&url, &name, &ext, pg, threshold).await;
        self.map.remove(id).await;

        result
    }

    async fn do_download(
        &self,
        url: &str,
        name: &str,
        ext: &str,
        pg: Arc<Progress>,
        threshold: Option<u64>,
    ) -> Result<()> {
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
                return Ok(());
            }
            temp.write_all(&chunk).await?;
            pg.progress(chunk.len() as u64);
        }

        // Check the size
        if let Some(threshold) = threshold {
            let size = pg.size();
            if size < threshold {
                anyhow::bail!("{} < {}", size, threshold);
            }
        }

        // Persist
        self.persist.persist(name, ext, &mut temp).await?;

        Ok(())
    }

    pub async fn index(&self) -> Vec<Item> {
        self.map.to_items().await
    }

    pub async fn abort(&self, id: Uuid) {
        self.map.abort(id).await;
    }
}
