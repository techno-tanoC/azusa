use reqwest::{Response, header};
use tokio::prelude::*;

use crate::progress::ProgressDecorator;
use crate::error::{Error, Result};

pub struct Download<'a, 'b, T> {
    res: &'a mut Response,
    deco: &'b mut ProgressDecorator<T>,
}

impl<'a, 'b, T: AsyncWrite + Unpin + Send> Download<'a, 'b, T> {
    pub fn new(res: &'a mut Response, deco: &'b mut ProgressDecorator<T>) -> Self {
        Download { res, deco }
    }

    pub async fn run(&mut self) -> Result<()> {
        if self.res.status().is_success() {
            if let Some(cl) = Self::content_length(self.res) {
                self.deco.set_total(cl);
            }
            Self::copy(&mut self.res, &mut self.deco).await
        } else {
            Err(Error::NonSuccessStatusError())
        }
    }
}

impl<'a, 'b, T> Download<'a, 'b, T> {
    async fn copy<W>(res: &mut Response, deco: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin + Send
    {
        while let Some(bytes) = res.chunk().await? {
            deco.write_all(&bytes).await?;
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
    use crate::progress::{Progress, ProgressDecorator};

    #[tokio::test]
    async fn run_without_content_length_test() {
        let pg = Progress::new("name");
        let mut deco = ProgressDecorator::new(pg.clone(), Cursor::new(vec![]));
        let body: reqwest::Body = vec![0, 1, 2].into();
        let mut res = http::response::Response::new(body).into();

        let item = pg.to_item("id");
        assert_eq!((item.size, item.total), (0, 0));

        Download::new(&mut res, &mut deco).run().await.unwrap();

        let item = pg.to_item("id");
        assert_eq!((item.size, item.total), (3, 0));
    }

    #[tokio::test]
    async fn run_with_content_length_test() {
        let pg = Progress::new("name");
        let mut deco = ProgressDecorator::new(pg.clone(), Cursor::new(vec![]));
        let body: reqwest::Body = vec![0, 1, 2].into();
        let mut res: reqwest::Response = http::response::Response::new(body).into();
        res.headers_mut().insert(header::CONTENT_LENGTH, "3".parse().unwrap());

        let item = pg.to_item("id");
        assert_eq!((item.size, item.total), (0, 0));

        Download::new(&mut res, &mut deco).run().await.unwrap();

        let item = pg.to_item("id");
        assert_eq!((item.size, item.total), (3, 3));
    }

    #[tokio::test]
    async fn copy_test() {
        let bytes = vec![0, 1, 2];
        let body: reqwest::Body = bytes.clone().into();
        let mut res = http::response::Response::new(body).into();
        let mut deco = Cursor::new(vec![]);
        let flag = Download::<()>::copy(&mut res, &mut deco).await.is_ok();
        assert!(flag);
        assert_eq!(deco.into_inner(), bytes);
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
