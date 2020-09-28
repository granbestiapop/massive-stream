use reqwest::{IntoUrl, Method, RequestBuilder, Response};
use std::time::Duration;

pub struct RestClient {
    inner: reqwest::Client,
}
pub struct ExponentialBackoff {
    timeout: u64,
    exp: usize,
    max_backoff: u64,
}

impl ExponentialBackoff {
    pub fn new(millisecs: u64) -> Self {
        Self {
            timeout: millisecs,
            exp: 1,
            max_backoff: 5000,
        }
    }
    pub fn time(&mut self) -> Duration {
        let timeout = self.max_backoff.min(self.exp as u64 * self.timeout);
        self.exp = self.exp * 2;
        Duration::from_millis(timeout)
    }
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
    pub async fn stream(&self, req: reqwest::Request) -> Result<Response, reqwest::Error> {
        self.inner.execute(req.try_clone().unwrap()).await
    }

    pub async fn execute(&self, req: reqwest::Request) -> Result<Response, reqwest::Error> {
        let mut tries: usize = 5;
        let mut backoff_strategy = ExponentialBackoff::new(50);

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
                    tokio::time::delay_for(backoff_strategy.time()).await;
                }
                res => return res,
            }
        }
    }
}
