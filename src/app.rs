use std::path::Path;
use tokio::fs::File;
use uuid::Uuid;

use super::download::Download;
use super::lock_copy::LockCopy;
use super::progress::Progress;
use super::table::Table;

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

    pub async fn download(&self, url: impl AsRef<str>, name: impl AsRef<str>, ext: impl AsRef<str>) {
        let id = Uuid::new_v4();
        let file = File::from_std(tempfile::tempfile().unwrap());
        let mut pg = Progress::new(name.as_ref(), file);
        self.table.add(id.to_string(), pg.clone()).await;

        let res = self.client.get(url.as_ref()).send().await.unwrap();
        let success = Download::new(res, pg.clone()).start().await;
        if success {
            self.lock_copy.rewind_copy(&mut pg, &name, &ext).await;
        }

        self.table.delete(id.to_string()).await;
    }
}