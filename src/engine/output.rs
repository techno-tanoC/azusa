use std::path::{Path, PathBuf};

use anyhow::Result;
use tokio::fs::File;

pub struct Output {
    path: PathBuf,
}

impl Output {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        Self { path }
    }

    pub async fn output(&self, src: &mut File, name: &str, ext: &str) -> Result<()> {
        let mut dst = self.fresh_file(name, ext).await?;
        tokio::io::copy(src, &mut dst).await?;
        Ok(())
    }

    async fn fresh_file(&self, name: &str, ext: &str) -> Result<File> {
        let mut fresh = None;
        for i in 0..10 {
            let basename = if i == 0 {
                format!("{name}.{ext}")
            } else {
                format!("{name}({i}).{ext}")
            };

            let mut path = self.path.clone();
            path.push(basename);
            dbg!(path.clone());
            let result = File::options()
                .write(true)
                .create_new(true)
                .open(path)
                .await;
            if let Ok(f) = result {
                fresh = Some(f);
                break;
            }
        }

        if let Some(f) = fresh {
            Ok(f)
        } else {
            Err(anyhow::anyhow!("too many file: {name}.{ext}"))
        }
    }
}
