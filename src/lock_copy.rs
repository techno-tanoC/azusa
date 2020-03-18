use std::borrow::Cow;
use std::io::SeekFrom;
use std::path::*;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{self, BufReader, BufWriter, AsyncSeek};
use tokio::prelude::*;
use tokio::sync::Mutex;

pub struct LockCopy(Arc<Mutex<PathBuf>>);

impl LockCopy {
    pub fn new<P: AsRef<Path>>(path: &P) -> Self {
        LockCopy(Arc::new(Mutex::new(path.as_ref().to_path_buf())))
    }

    pub async fn rewind_copy<R, S, T>(&self, from: &mut R, name: &S, ext: &T)
    where
        R: AsyncRead + AsyncSeek + Unpin + Send,
        S: AsRef<str>,
        T: AsRef<str>,
    {
        let s = self.0.lock().await;
        let fresh = Self::fresh_name(&*s, name, ext);
        let dest = File::create(fresh).await.unwrap();
        from.seek(SeekFrom::Start(0));
        let (mut from, mut dest) = (BufReader::new(from), BufWriter::new(dest));
        io::copy(&mut from, &mut dest).await.unwrap();
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
