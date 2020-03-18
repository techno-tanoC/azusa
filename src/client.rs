use futures::stream::TryStreamExt;
use reqwest::{Response, header};
use tokio::io;
use tokio::prelude::*;
use tokio_util::compat::FuturesAsyncReadCompatExt;

use super::progress::Progress;

pub struct Client<'a, T> {
    client: &'a reqwest::Client,
    res: Response,
    pg: Progress<T>,
}

impl<'a, T: AsyncWrite + Unpin + Send> Client<'a, T> {
    pub fn new<U>(client: &'a reqwest::Client, res: Response, pg: Progress<T>) -> Self
    where
        U: AsRef<str>,
    {
        Client { client, res, pg }
    }

    pub async fn start(mut self) -> Progress<T> {
        if self.res.status().is_success() {
            if let Some(cl) = Self::content_length(&self.res) {
                self.pg.set_total(cl);
            }
            let Client { res, mut pg, .. } = self;
            let mut stream = res
                .bytes_stream()
                .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
                .into_async_read()
                .compat();
            Self::copy(&mut stream, &mut pg).await;
            pg
        } else {
            self.pg
        }
    }

    async fn copy<R, W>(reader: &mut R, writer: &mut W)
    where
        R: AsyncRead + Unpin + Send,
        W: AsyncWrite + Unpin + Send,
    {
        let result = io::copy(reader, writer).await;
    }

    fn content_length(res: &Response) -> Option<u64> {
        res.headers()
            .get(header::CONTENT_LENGTH)?
            .to_str().ok()?.parse().ok()
    }
}
