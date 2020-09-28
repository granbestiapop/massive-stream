use reqwest::{Client, IntoUrl, Method, Request, RequestBuilder, Response};
use std::time::Duration;

pub struct RestClient {
    inner: reqwest::Client,
}

impl RestClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        let inner = reqwest::Client::builder().build()?;

        Ok(Self { inner })
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        self.inner.request(method, url)
    }

    // TODO with header
    pub async fn stream(&self, req: reqwest::Request) -> Result<Response, reqwest::Error>{
        self.inner.execute(req.try_clone().unwrap()).await
    }

    pub async fn execute(
        &self,
        req: reqwest::Request,
    ) -> Result<Response, reqwest::Error> {
        let mut tries: usize = 5;

        loop {
            let res = self
                .inner
                .execute(req.try_clone().unwrap())
                .await
                .and_then(|r| r.error_for_status());
            match res {
                Err(e) if tries > 1 => {
                    tries -= 1;
                    //log::error!("{}", e);
                    tokio::time::delay_for(Duration::from_millis(500)).await;
                }
                res => return res,
            }
        }
    }
}
