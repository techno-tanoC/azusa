use futures::future::FutureExt;
use std::io::SeekFrom;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Poll, Context};
use tokio::io::{AsyncSeek, Result, ErrorKind};
use tokio::prelude::*;
use tokio::sync::Mutex;

use super::item::Item;

struct ProgressInner<T> {
    name: String,
    total: u64,
    size: u64,
    canceled: bool,
    buf: T,
}

#[derive(Clone)]
pub struct Progress<T> {
    inner: Arc<Mutex<ProgressInner<T>>>,
}

impl<T> Progress<T> {
    pub fn new(name: impl ToString, buf: T) -> Self {
        let inner = Arc::new(Mutex::new(ProgressInner {
            name: name.to_string(),
            total: 0,
            size: 0,
            canceled: false,
            buf,
        }));
        Progress { inner }
    }

    pub async fn set_total(&mut self, total: u64) {
        self.inner.lock().await.total = total;
    }

    pub async fn cancel(&mut self) {
        self.inner.lock().await.canceled = true
    }

    pub async fn to_item(&self, id: impl ToString) -> Item {
        let pg = self.inner.lock().await;
        Item {
            id: id.to_string(),
            name: pg.name.clone(),
            total: pg.total,
            size: pg.size,
            canceled: pg.canceled,
        }
    }
}

impl<T: AsyncRead + Unpin + Send> AsyncRead for Progress<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8]
    ) -> Poll<Result<usize>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                Pin::new(&mut s.buf).poll_read(cx, buf)
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }
}

impl<T: AsyncWrite + Unpin + Send> AsyncWrite for Progress<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8]
    ) -> Poll<Result<usize>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                if s.canceled {
                    Poll::Ready(Err(io::Error::new(ErrorKind::Interrupted, "canceled")))
                } else {
                    let poll = Pin::new(&mut s.buf).poll_write(cx, buf);
                    if let Poll::Ready(Ok(n)) = poll {
                        s.size += n as u64;
                    }
                    poll
                }
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<()>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                Pin::new(&mut s.buf).poll_flush(cx)
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<()>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                Pin::new(&mut s.buf).poll_shutdown(cx)
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }
}

impl<T: AsyncSeek + Unpin + Send> AsyncSeek for Progress<T> {
    fn start_seek(
        self: Pin<&mut Self>,
        cx: &mut Context,
        position: SeekFrom
    ) -> Poll<Result<()>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                Pin::new(&mut s.buf).start_seek(cx, position)
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }

    fn poll_complete(
        self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<u64>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                Pin::new(&mut s.buf).poll_complete(cx)
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::io::AsyncReadExt;
    use std::io::Cursor;

    #[tokio::test]
    async fn set_total_test() {
        let mut pg = Progress::new("name", ());
        assert_eq!(pg.to_item("").await.total, 0);
        pg.set_total(1000).await;
        assert_eq!(pg.to_item("").await.total, 1000);
    }

    #[tokio::test]
    async fn cancel_test() {
        let mut pg = Progress::new("name", ());
        assert_eq!(pg.to_item("").await.canceled, false);
        pg.cancel().await;
        assert_eq!(pg.to_item("").await.canceled, true);
    }

    #[tokio::test]
    async fn to_item_test() {
        let pg = Progress {
            inner: Arc::new(Mutex::new(
                ProgressInner {
                    name: "name".to_string(),
                    total: 1000,
                    size: 200,
                    canceled: true,
                    buf: (),
                }
        ))};

        let item = Item {
            id: "id".to_string(),
            name: "name".to_string(),
            total: 1000,
            size: 200,
            canceled: true,
        };

        assert_eq!(pg.to_item("id").await, item);
    }

    #[tokio::test]
    async fn async_read_test() {
        let mut pg = Progress::new("name", Cursor::new(vec![0, 1, 2]));
        let mut buf = vec![];
        let n = pg.read_to_end(&mut buf).await.unwrap();
        assert_eq!(n , 3);
        assert_eq!(buf, vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn async_write_test() {
        let mut pg = Progress::new("name", Cursor::new(vec![]));
        let buf = [0, 1, 2];
        pg.write_all(&buf).await.unwrap();
        assert_eq!(pg.inner.lock().await.buf.get_ref(), &vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn async_seek_test() {
        let mut pg = Progress::new("name", Cursor::new(vec![0, 1, 2]));
        let mut buf = vec![];

        let n = pg.read_to_end(&mut buf).await.unwrap();
        assert_eq!(n , 3);
        assert_eq!(buf, vec![0, 1, 2]);

        pg.seek(SeekFrom::Start(0)).await.unwrap();

        let n = pg.read_to_end(&mut buf).await.unwrap();
        assert_eq!(n , 3);
        assert_eq!(buf, vec![0, 1, 2, 0, 1, 2]);
    }

    #[tokio::test]
    async fn progress_test() {
        let mut pg = Progress::new("name", Cursor::new(vec![]));
        let mut buf = vec![0, 1, 2];

        assert_eq!(pg.inner.lock().await.size, 0);
        pg.write_all(&mut buf).await.unwrap();
        assert_eq!(pg.inner.lock().await.size, 3);
        pg.write_all(&mut buf).await.unwrap();
        assert_eq!(pg.inner.lock().await.size, 6);
    }

    #[tokio::test]
    async fn cancel_write_test() {
        let mut pg = Progress::new("name", Cursor::new(vec![]));
        let mut buf = vec![0, 1, 2];

        assert_eq!(pg.inner.lock().await.canceled, false);
        assert!(pg.write_all(&mut buf).await.is_ok());

        pg.cancel().await;

        assert_eq!(pg.inner.lock().await.canceled, true);
        assert!(pg.write_all(&mut buf).await.is_err());
    }
}
