use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt as _},
};

use crate::{client::DataClient, progress::Progress, table::Table};

pub struct Download<C> {
    volume: PathBuf,
    client: C,
    table: Table,
}

impl<C: DataClient> Download<C> {
    pub fn new(volume: PathBuf, client: C) -> Result<Self> {
        let table = Table::new();
        Ok(Self {
            volume,
            client,
            table,
        })
    }
}

impl<C> Download<C>
where
    C: DataClient,
{
    pub async fn start(&self, url: &str, name: &str, ext: &str) -> Result<()> {
        let pg = Arc::new(Progress::new(name));
        let id = self.table.add(pg.clone()).await;

        let result = self.download(url, name, ext, pg).await;

        // When paniced, it is not necessary to delete it.
        self.table.delete(id).await;
        result
    }

    async fn download(&self, url: &str, name: &str, ext: &str, pg: Arc<Progress>) -> Result<()> {
        let mut file = File::from_std(tempfile::tempfile()?);
        let mut response = self.client.get(url).await?;

        // Set Content-Length
        if let Some(content_length) = response.content_length() {
            pg.set_total(content_length);
        }

        // Copy bytes from response body to progress and file
        while let Some(mut chunk) = response.chunk().await? {
            if pg.is_canceled() {
                anyhow::bail!("canceled: {name}");
            }
            pg.progress(chunk.len() as u64);
            file.write_all_buf(&mut chunk).await?;
            println!("{:?}", pg);
        }

        // Need rewind for copying
        file.rewind().await?;

        let mut dest = File::options()
            .write(true)
            .create_new(true)
            .open(format!("{name}.{ext}"))
            .await?;
        tokio::io::copy(&mut file, &mut dest).await?;

        println!("{:?}", pg);

        Ok(())
    }
}
