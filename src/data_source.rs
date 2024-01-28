use std::time::Duration;

use anyhow::Result;
use reqwest::{Client, Response};

pub trait DataSource {
    async fn get(&self, url: &str) -> Result<Response>;
}

#[derive(Debug, Clone)]
pub struct HttpSource {
    client: Client,
}

impl HttpSource {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self { client })
    }
}

impl DataSource for HttpSource {
    async fn get(&self, url: &str) -> Result<Response> {
        let response = self.client.get(url).send().await?;
        Ok(response)
    }
}

#[cfg(test)]
pub mod test {
    use reqwest::{Body, StatusCode};

    use super::*;

    pub struct TestSource<T> {
        status: u16,
        body: T,
    }

    impl<T> TestSource<T> {
        pub fn new(status: u16, body: T) -> Result<Self> {
            Ok(Self { status, body })
        }
    }

    impl<T> DataSource for TestSource<T>
    where
        T: Into<Body> + Clone,
    {
        async fn get(&self, _url: &str) -> Result<Response> {
            let body = self.body.clone().into();
            let response = http::response::Builder::new()
                .status(self.status)
                .body(body)?;
            Ok(response.into())
        }
    }

    #[tokio::test]
    async fn test_client() {
        let body = "hello world".to_string();
        let client = TestSource::new(200, body.clone()).unwrap();
        let response = client.get("http://dummy.com/").await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.content_length(), Some(11));
        assert_eq!(response.text().await.unwrap(), body);
    }
}
