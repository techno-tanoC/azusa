use futures::future::FutureExt;
use std::fmt;
use std::io::SeekFrom;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Poll, Context};
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncSeek, Result, ErrorKind};
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

    pub async fn set_total(&mut self) -> u64 {
        self.inner.lock().await.total
    }

    pub async fn cancele(&mut self) {
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
