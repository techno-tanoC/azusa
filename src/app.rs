use std::time::Duration;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncSeek;
use tokio::prelude::*;
use uuid::Uuid;

use crate::download::Download;
use crate::lock_copy::LockCopy;
use crate::progress::{Progress, ProgressDecorator};
use crate::table::Table;
use crate::error::Result;

#[derive(Clone)]
pub struct App {
    pub client: reqwest::Client,
    pub lock_copy: LockCopy,
    pub table: Table<String, Arc<Progress>>,
}

impl App {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(30))
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed ClientBuilder::build()");
        let lock_copy = LockCopy::new(&path);
        let table = Table::new();
        App { client, lock_copy, table }
    }

    pub async fn download(&self, url: impl AsRef<str>, name: impl AsRef<str>, ext: impl AsRef<str>) -> Result<()> {
        let id = Uuid::new_v4();

        debug!("app::download id: {:?} url: {:?} name: {:?} ext: {:?}", id, url.as_ref(), name.as_ref(), ext.as_ref());

        let file = tempfile::tempfile()?;
        let file = File::from_std(file);
        let pg = Progress::new(name.as_ref());
        self.table.add(id.to_string(), pg.clone()).await;
        let mut deco = ProgressDecorator::new(pg, file);
        let ret = self.do_download(&mut deco, url, name, ext).await;
        self.table.delete(id.to_string()).await;
        ret
    }

    async fn do_download<T>(&self, deco: &mut ProgressDecorator<T>, url: impl AsRef<str>, name: impl AsRef<str>, ext: impl AsRef<str>) -> Result<()>
    where
        T: AsyncRead + AsyncWrite + AsyncSeek + Unpin + Send,
    {
        let mut res = self.client.get(url.as_ref()).send().await?;
        let ret = Download::new(&mut res, deco).run().await;
        if ret.is_ok() {
            self.lock_copy.copy(deco, &name, &ext).await
        } else {
            ret
        }
    }
}
