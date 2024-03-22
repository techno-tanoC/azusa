use anyhow::Result;
use uuid::fmt::Hyphenated;

pub struct Engine;

impl Engine {
    pub fn build() -> Self {
        Engine
    }

    pub async fn index(&self) -> Vec<()> {
        vec![]
    }

    pub async fn start(&self, url: &str, name: &str, ext: &str) -> Result<()> {
        Ok(())
    }

    pub async fn cancel(&self, id: Hyphenated) -> Result<()> {
        Ok(())
    }
}
