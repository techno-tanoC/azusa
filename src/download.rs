use reqwest::{Response, header};
use tokio::prelude::*;

use crate::progress::Progress;
use crate::error::{Error, Result};

pub struct Download<'a, T> {
    res: &'a mut Response,
    pg: Progress<T>,
}

impl<'a, 'b, T: AsyncWrite + Unpin + Send> Download<'a, T> {
    pub fn new(res: &'a mut Response, pg: Progress<T>) -> Self {
        Download { res, pg }
    }

    pub async fn run(&mut self) -> Result<()> {
        if self.res.status().is_success() {
            if let Some(cl) = Self::content_length(self.res) {
                self.pg.set_total(cl).await;
            }
            Self::copy(&mut self.res, &mut self.pg).await
        } else {
            Err(Error::NonSuccessStatusError())
        }
    }
}

impl<'a, T> Download<'a, T> {
    async fn copy<W>(res: &mut Response, writer: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin + Send
    {
        while let Some(bytes) = res.chunk().await? {
            writer.write_all(&bytes).await?;
        }
        Ok(())
    }

    fn content_length(res: &Response) -> Option<u64> {
        res.headers()
            .get(header::CONTENT_LENGTH)?
            .to_str().ok()?.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;
    use crate::progress::Progress;

    #[tokio::test]
    async fn run_without_content_length_test() {
        let pg = Progress::new("name", Cursor::new(vec![]));
        let body: reqwest::Body = vec![0, 1, 2].into();
        let mut res = http::response::Response::new(body).into();

        let item = pg.to_item("id").await;
        assert_eq!((item.size, item.total), (0, 0));

        Download::new(&mut res, pg.clone()).run().await.unwrap();

        let item = pg.to_item("id").await;
        assert_eq!((item.size, item.total), (3, 0));
    }

    #[tokio::test]
    async fn run_with_content_length_test() {
        let pg = Progress::new("name", Cursor::new(vec![]));
        let body: reqwest::Body = vec![0, 1, 2].into();
        let mut res: reqwest::Response = http::response::Response::new(body).into();
        res.headers_mut().insert(header::CONTENT_LENGTH, "3".parse().unwrap());

        let item = pg.to_item("id").await;
        assert_eq!((item.size, item.total), (0, 0));

        Download::new(&mut res, pg.clone()).run().await.unwrap();

        let item = pg.to_item("id").await;
        assert_eq!((item.size, item.total), (3, 3));
    }

    #[tokio::test]
    async fn copy_test() {
        let bytes = vec![0, 1, 2];
        let body: reqwest::Body = bytes.clone().into();
        let mut res = http::response::Response::new(body).into();
        let mut writer = Cursor::new(vec![]);
        let flag = Download::<()>::copy(&mut res, &mut writer).await.is_ok();
        assert!(flag);
        assert_eq!(writer.into_inner(), bytes);
    }

    #[test]
    fn content_length_test() {
        let body: reqwest::Body = vec![].into();
        let res = http::response::Response::new(body).into();
        assert!(Download::<()>::content_length(&res).is_none());

        let body: reqwest::Body = vec![].into();
        let mut res: reqwest::Response = http::response::Response::new(body).into();
        res.headers_mut().insert(header::CONTENT_LENGTH, "invalid".parse().unwrap());
        assert!(Download::<()>::content_length(&res).is_none());

        let body: reqwest::Body = vec![].into();
        let mut res: reqwest::Response = http::response::Response::new(body).into();
        res.headers_mut().insert(header::CONTENT_LENGTH, "1000".parse().unwrap());
        assert_eq!(Download::<()>::content_length(&res).unwrap(), 1000);
    }
}
