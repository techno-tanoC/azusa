use futures::stream::TryStreamExt;
use reqwest::{Response, header};
use tokio::io;
use tokio::prelude::*;
use tokio_util::compat::FuturesAsyncReadCompatExt;

use super::progress::Progress;

pub struct Download<T> {
    res: Response,
    pg: Progress<T>,
}

impl<T: AsyncWrite + Unpin + Send> Download<T> {
    pub fn new(res: Response, pg: Progress<T>) -> Self {
        Download { res, pg }
    }

    pub async fn start(mut self) {
        if self.res.status().is_success() {
            if let Some(cl) = Self::content_length(&self.res) {
                self.pg.set_total(cl).await;
            }
            let Download { res, mut pg, .. } = self;
            let mut stream = res
                .bytes_stream()
                .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
                .into_async_read()
                .compat();
            Self::copy(&mut stream, &mut pg).await;
        }
    }

    async fn copy<R, W>(reader: &mut R, writer: &mut W)
    where
        R: AsyncRead + Unpin + Send,
        W: AsyncWrite + Unpin + Send,
    {
        io::copy(reader, writer).await.unwrap();
    }

    fn content_length(res: &Response) -> Option<u64> {
        res.headers()
            .get(header::CONTENT_LENGTH)?
            .to_str().ok()?.parse().ok()
    }
}
