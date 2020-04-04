use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncSeek;
use tokio::prelude::*;
use uuid::Uuid;

use crate::download::Download;
use crate::lock_copy::LockCopy;
use crate::progress::Progress;
use crate::table::Table;
use crate::error::Result;

#[derive(Clone)]
pub struct App {
    pub client: reqwest::Client,
    pub lock_copy: LockCopy,
    pub table: Table<String, Progress<File>>,
}

impl App {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed ClientBuilder::build()");
        let lock_copy =LockCopy::new(&path);
        let table = Table::new();
        App { client, lock_copy, table }
    }

    pub async fn download(&self, url: impl AsRef<str>, name: impl AsRef<str>, ext: impl AsRef<str>) -> Result<()> {
        let id = Uuid::new_v4();
        let file = File::from_std(tempfile::tempfile()?);
        let mut pg = Progress::new(name.as_ref(), file);
        self.table.add(id.to_string(), pg.clone()).await;
        let ret = self.do_download(&mut pg, url, name, ext).await;
        self.table.delete(id.to_string()).await;
        ret
    }

    async fn do_download<T>(&self, pg: &mut Progress<T>, url: impl AsRef<str>, name: impl AsRef<str>, ext: impl AsRef<str>) -> Result<()>
    where
        T: AsyncRead + AsyncWrite + AsyncSeek + Unpin + Send,
    {
        let res = self.client.get(url.as_ref()).send().await?;
        let ret = Download::new(res, pg.clone()).run().await;
        if let Ok(()) = ret {
            self.lock_copy.copy(pg, &name, &ext).await
        } else {
            ret
        }
    }
}
