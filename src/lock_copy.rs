use std::borrow::Cow;
use std::io::SeekFrom;
use std::path::*;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{self, BufReader, BufWriter, AsyncSeek};
use tokio::prelude::*;
use tokio::sync::Mutex;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct LockCopy(Arc<Mutex<PathBuf>>);

impl LockCopy {
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        LockCopy(Arc::new(Mutex::new(path.as_ref().to_path_buf())))
    }

    pub async fn copy<R, S, T>(&self, from: &mut R, name: &S, ext: &T) -> Result<()>
    where
        R: AsyncRead + AsyncSeek + Unpin + Send,
        S: AsRef<str>,
        T: AsRef<str>,
    {
        debug!("lock_copy::copy name: {:?} ext: {:?}", name.as_ref(), ext.as_ref());

        let s = self.0.lock().await;
        let fresh = Self::fresh_name(&*s, name, ext);
        let mut dest = File::create(fresh).await?;
        Ok(())
    }

    async fn rewind_copy<R, W>(reader: &mut R, writer: &mut W) -> Result<()>
    where
        R: AsyncRead + AsyncSeek + Unpin + Send,
        W: AsyncWrite + Unpin + Send,
    {
        reader.seek(SeekFrom::Start(0)).await?;
        let (mut reader, mut writer) = (BufReader::new(reader), BufWriter::new(writer));
        io::copy(&mut reader, &mut writer).await?;
        Ok(())
    }

    fn fresh_name<P, S, T>(path: &P, name: &S, ext: &T) -> PathBuf
    where
        P: AsRef<Path>,
        S: AsRef<str>,
        T: AsRef<str>,
    {
        let mut i = 0;
        loop {
            let name = Self::build_name(name, i, ext);
            let candidate = path.as_ref().join(name);
            if candidate.exists() {
                i += 1;
            } else {
                return candidate.to_path_buf();
            }
        }
    }

    fn build_name<S, T>(name: &S, count: u64, ext: &T) -> String
    where
        S: AsRef<str>,
        T: AsRef<str>,
    {
        let count: Cow<'_, _> = if count >= 1 {
            format!("({})", count).into()
        } else {
            "".into()
        };

        let ext: Cow<'_, _> = if ext.as_ref().is_empty() {
            "".into()
        } else {
            format!(".{}", ext.as_ref()).into()
        };

        name.as_ref().to_string() + &count + &ext
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[tokio::test]
    async fn copy_test() {
        let dir = tempfile::tempdir().unwrap();
        let lock_copy = LockCopy::new(&dir);
        let mut from = File::open("Cargo.lock").await.unwrap();
        let (name, ext) = ("test", "lock");
        lock_copy.copy(&mut from, &name, &ext).await.unwrap();
    }

    #[tokio::test]
    async fn rewind_copy_test() {
        let mut from = Cursor::new(vec![]);
        from.write_all(&[0u8, 1, 2]).await.unwrap();
        let mut dest = Cursor::new(vec![]);
        LockCopy::rewind_copy(&mut from, &mut dest).await.unwrap();
        assert_eq!(from.position(), 3);
        assert_eq!(dest.into_inner(), vec![0, 1, 2]);
    }

    #[test]
    fn test_fresh_name() {
        let fresh = LockCopy::fresh_name(&".", &"dummy", &"toml");
        assert_eq!(fresh.to_string_lossy(), "./dummy.toml".to_string());

        let fresh = LockCopy::fresh_name(&".", &"Cargo", &"toml");
        assert_eq!(fresh.to_string_lossy(), "./Cargo(1).toml".to_string());
    }

    #[test]
    fn test_build_name() {
        let name = LockCopy::build_name(&"hello", 0, &"jpg");
        assert_eq!(name, "hello.jpg");

        let name = LockCopy::build_name(&"hello", 1, &"jpg");
        assert_eq!(name, "hello(1).jpg");

        let name = LockCopy::build_name(&"hello", 0, &"");
        assert_eq!(name, "hello");

        let name = LockCopy::build_name(&"hello", 1, &"");
        assert_eq!(name, "hello(1)");
    }
}
