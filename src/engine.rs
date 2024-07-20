pub mod item;
mod progress;
mod progress_map;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt as _},
};
use uuid::Uuid;

use item::Item;
use progress::Progress;
use progress_map::ProgressMap;

pub struct Engine {
    client: reqwest::Client,
    map: ProgressMap,
    dest_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Success,
    Cancel,
}

impl Engine {
    pub fn new(client: reqwest::Client, dest_path: PathBuf) -> Self {
        Self {
            client,
            map: ProgressMap::default(),
            dest_path,
        }
    }

    pub async fn download(&self, url: String, title: String, ext: String) -> Result<Status> {
        let id = Uuid::now_v7();
        let pg = Arc::new(Progress::new(url, title, ext));
        let _guard = self.map.add(id, pg.clone()).await;

        let temp = tempfile::tempfile()?;
        let mut file = tokio::fs::File::from_std(temp);

        let status = self.do_download(pg.clone(), &mut file).await?;
        if status == Status::Success {
            Self::persist(&mut file, pg.title(), pg.ext(), &self.dest_path).await
        } else {
            Ok(status)
        }
    }

    async fn do_download(&self, pg: Arc<Progress>, file: &mut File) -> Result<Status> {
        let mut response = self.client.get(pg.url()).send().await?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!(
                "status code is not success: {} {:?} {:?}",
                status,
                pg.title(),
                pg.url()
            );
        }

        if let Some(content_length) = response.content_length() {
            pg.set_total(content_length);
        }

        while let Some(chunk) = response.chunk().await? {
            if pg.is_canceled() {
                println!("cancelled: {:?} {:?}", pg.title(), pg.url());
                return Ok(Status::Cancel);
            }
            pg.progress(chunk.len() as u64);
            file.write_all(&chunk).await?;

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        Ok(Status::Success)
    }

    async fn persist(temp: &mut File, title: &str, ext: &str, dest_path: &Path) -> Result<Status> {
        temp.rewind().await?;

        let mut count = 0;
        loop {
            if count >= 10 {
                anyhow::bail!("too many files");
            }
            let name = if count == 0 {
                format!("{title}.{ext}")
            } else {
                format!("{title}({count}).{ext}")
            };
            count += 1;

            let dest = dest_path.join(name);
            if let Ok(mut file) = File::create_new(dest).await {
                tokio::io::copy(temp, &mut file).await?;
                break;
            }
        }
        Ok(Status::Success)
    }

    pub async fn to_items(&self) -> Vec<Item> {
        self.map.to_items().await
    }

    pub async fn cancel(&self, id: Uuid) {
        self.map.cancel(id).await;
        self.map.remove(id).await;
    }
}
