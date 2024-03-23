use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use uuid::fmt::Hyphenated;

pub struct Engine {
    client: Client,
}

impl Engine {
    pub fn build() -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self { client })
    }

    pub async fn index(&self) -> Vec<()> {
        vec![]
    }

    pub async fn start(&self, url: &str, name: &str, ext: &str) -> Result<()> {
        let response = self.client.get(url).send().await?;
        Ok(())
    }

    pub async fn cancel(&self, id: Hyphenated) -> Result<()> {
        Ok(())
    }
}
