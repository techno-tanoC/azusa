mod output;

use std::{path::Path, time::Duration};

use anyhow::Result;
use reqwest::Client;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use uuid::fmt::Hyphenated;

use self::output::Output;

pub struct Engine {
    client: Client,
    output: Output,
}

impl Engine {
    pub fn build(path: impl AsRef<Path>) -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .build()?;
        let output = Output::new(path.as_ref());
        Ok(Self { client, output })
    }

    pub async fn index(&self) -> Vec<()> {
        vec![]
    }

    pub async fn start(&self, url: &str, name: &str, ext: &str) -> Result<()> {
        let mut temp =
            tokio::task::spawn_blocking(|| tempfile::tempfile().map(tokio::fs::File::from_std))
                .await??;
        let mut response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            Err(anyhow::anyhow!(
                "Response status is not success: {name} {ext} {url}"
            ))?;
        }

        while let Some(mut chunk) = response.chunk().await? {
            temp.write_all_buf(&mut chunk).await?;
        }

        // Rewind for copy
        temp.rewind().await?;

        // Persist the file
        self.output.output(&mut temp, name, ext).await?;

        Ok(())
    }

    pub async fn cancel(&self, id: Hyphenated) -> Result<()> {
        Ok(())
    }
}
