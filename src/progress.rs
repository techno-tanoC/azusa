use serde::Serialize;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Poll, Context};
use tokio::io::{Result, ErrorKind};
use tokio::prelude::*;

use crate::item::Item;

#[derive(Serialize, Debug)]
pub struct Progress {
    name: String,
    total: AtomicU64,
    size: AtomicU64,
    canceled: AtomicBool,
}

impl Progress {
    pub fn new(name: impl ToString) -> Arc<Self> {
        Arc::new(Progress {
            name: name.to_string(),
            total: 0.into(),
            size: 0.into(),
            canceled: false.into(),
        })
    }

    pub fn cancel(&self) {
        self.canceled.store(true, Ordering::SeqCst);
    }

    pub fn to_item(&self, id: impl ToString) -> Item {
        Item {
            id: id.to_string(),
            name: self.name.clone(),
            total: self.total.load(Ordering::SeqCst),
            size: self.size.load(Ordering::SeqCst),
            canceled: self.canceled.load(Ordering::SeqCst),
        }
    }
}

pub struct ProgressWriter<T> {
    pg: Arc<Progress>,
    buf: T,
}

impl<T> ProgressWriter<T> {
    pub fn new(pg: Arc<Progress>, buf: T) -> Self {
        ProgressWriter {
            pg,
            buf,
        }
    }

    pub fn set_total(&mut self, total: u64) {
        self.pg.total.store(total, Ordering::SeqCst);
    }
}

impl<T: AsyncWrite + Unpin + Send> AsyncWrite for ProgressWriter<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8]
    ) -> Poll<Result<usize>> {
        if self.pg.canceled.load(Ordering::SeqCst) {
            Poll::Ready(Err(io::Error::new(ErrorKind::Interrupted, "canceled")))
        } else {
            let poll = Pin::new(&mut self.buf).poll_write(cx, buf);
            if let Poll::Ready(Ok(n)) = poll {
                self.pg.size.fetch_add(n as u64, Ordering::SeqCst);
            }
            poll
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.buf).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.buf).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn set_total_test() {
        let mut writer = ProgressWriter::new(Progress::new("name"), ());
        assert_eq!(writer.pg.to_item("").total, 0);
        writer.set_total(1000);
        assert_eq!(writer.pg.to_item("").total, 1000);
    }

    #[test]
    fn cancel_test() {
        let writer = ProgressWriter::new(Progress::new("name"), ());
        assert_eq!(writer.pg.to_item("").canceled, false);
        writer.pg.cancel();
        assert_eq!(writer.pg.to_item("").canceled, true);
    }

    #[test]
    fn to_item_test() {
        let writer = ProgressWriter {
            pg: Arc::new(Progress {
                name: "name".to_string(),
                total: 1000.into(),
                size: 200.into(),
                canceled: true.into(),
            }),
            buf: (),
        };

        let item = Item {
            id: "id".to_string(),
            name: "name".to_string(),
            total: 1000,
            size: 200,
            canceled: true,
        };

        assert_eq!(writer.pg.to_item("id"), item);
    }

    #[tokio::test]
    async fn async_write_test() {
        let mut writer = ProgressWriter::new(Progress::new("name"), Cursor::new(vec![]));
        let buf = [0, 1, 2];
        writer.write_all(&buf).await.unwrap();
        assert_eq!(writer.buf.get_ref(), &vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn progress_test() {
        let mut writer = ProgressWriter::new(Progress::new("name"), Cursor::new(vec![]));
        let mut buf = vec![0, 1, 2];

        assert_eq!(writer.pg.size.load(Ordering::SeqCst), 0);
        writer.write_all(&mut buf).await.unwrap();
        assert_eq!(writer.pg.size.load(Ordering::SeqCst), 3);
        writer.write_all(&mut buf).await.unwrap();
        assert_eq!(writer.pg.size.load(Ordering::SeqCst), 6);
    }

    #[tokio::test]
    async fn cancel_write_test() {
        let mut writer = ProgressWriter::new(Progress::new("name"), Cursor::new(vec![]));
        let mut buf = vec![0, 1, 2];

        assert_eq!(writer.pg.canceled.load(Ordering::SeqCst), false);
        assert!(writer.write_all(&mut buf).await.is_ok());

        writer.pg.cancel();

        assert_eq!(writer.pg.canceled.load(Ordering::SeqCst), true);
        assert!(writer.write_all(&mut buf).await.is_err());
    }
}
