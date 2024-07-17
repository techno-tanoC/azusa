use std::sync::Arc;

use anyhow::Result;

use super::progress::Progress;

pub trait ClientLike {
    fn get(&self, url: &str)
        -> impl std::future::Future<Output = Result<reqwest::Response>> + Send;
}

impl ClientLike for reqwest::Client {
    async fn get(&self, url: &str) -> Result<reqwest::Response> {
        let response = self.get(url).send().await?;
        Ok(response)
    }
}

pub struct Downloader<C> {
    client: C,
}

impl<C> Downloader<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

impl<C: ClientLike> Downloader<C> {
    pub async fn start(&self, pg: Arc<Progress>) -> Result<()> {
        let mut response = self.client.get(pg.url()).await?;

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
                anyhow::bail!("cancelled: {:?} {:?}", pg.title(), pg.url())
            }
            pg.progress(chunk.len() as u64);
        }

        Ok(())
    }
}
