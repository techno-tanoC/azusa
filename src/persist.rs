use anyhow::Result;
use std::path::PathBuf;

use tokio::{fs::File, io::AsyncSeekExt};

#[derive(Debug)]
pub struct Persist {
    dest: PathBuf,
}

impl Persist {
    pub fn new(dest: PathBuf) -> Self {
        Self { dest }
    }

    pub async fn persist(&self, name: &str, ext: &str, src: &mut File) -> Result<()> {
        let mut dst = self.fresh_file(name, ext).await?;
        src.rewind().await?;
        tokio::io::copy(src, &mut dst).await?;
        Ok(())
    }

    async fn fresh_file(&self, name: &str, ext: &str) -> Result<File> {
        const MAX_ATTEMPTS: usize = 10;

        for i in 0..MAX_ATTEMPTS {
            let fresh_name = if i == 0 {
                format!("{name}.{ext}")
            } else {
                format!("{name}({i}).{ext}")
            };
            let path = self.dest.join(fresh_name);
            if let Ok(file) = File::create_new(path).await {
                return Ok(file);
            }
        }

        anyhow::bail!("too many retry")
    }
}
