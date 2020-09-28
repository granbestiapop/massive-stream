// use tokio::stream::StreamExt as _;
use tokio::fs::File;

use futures::stream::{StreamExt, TryStreamExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::compat::FuturesAsyncReadCompatExt;

// MOve
//use tokio_retry::Retry;
//use tokio_retry::strategy::{ExponentialBackoff, jitter};
use reqwest::{Client, IntoUrl, Method, Request, RequestBuilder};
use std::time::Duration;

const CONCURRENT_REQUESTS: usize = 3;
const S3_FILE_URL: &str = "http://localhost:8080/s3";
const UPLOADER_DATA_PATH: &str = "http://localhost:8080/a/";
const LOCAL_FILE_DIR: &str = "data/foo.txt";

async fn do_call(client: &RestClient, path: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}{}", UPLOADER_DATA_PATH, path);
    println!("Starting{}", url);
    let req = client
        .request(reqwest::Method::GET, url.as_str())
        .build()?;
    let text = client.execute(req).await?.text().await?;
    Ok(text)
}

pub struct RestClient {
    inner: reqwest::Client,
}

impl RestClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        let inner = reqwest::Client::builder()
            .build()?;

        Ok(Self { inner })
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        self.inner.request(method, url)
    }

    pub async fn execute(
        &self,
        req: reqwest::Request,
    ) -> Result<reqwest::Response, reqwest::Error> {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let file = File::open(LOCAL_FILE_DIR).await?;
    let file_stream = BufReader::new(file);
    let client = Client::new();
    let chunk_size: usize = 1024 * 1024; // 1M
    let mut total_chunks: u32 = 0;
    let mut bytes_size: usize = 0;

    let restclient = RestClient::new()?;

    let response = client.get(S3_FILE_URL).send().await?;
    response
        //# Input Stream
        .bytes_stream()
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read()
        .compat();

    file_stream
        // Split chunks into lines
        .lines()
        //# Transformer Stream
        .map(Result::unwrap)
        // Update chunks and return line
        .map(|line| {
            let size = line.bytes().len();
            bytes_size += size;
            if bytes_size >= chunk_size {
                bytes_size -= chunk_size;
                total_chunks += 1;
            }
            println!("chunks: {}, bytes:{}", total_chunks, bytes_size);
            line
        })
        //# Output Stream
        // Do api calls to external service
        .map(|st| do_call(&restclient, st))
        // Execute on schduler with max concurrent requests
        .buffer_unordered(CONCURRENT_REQUESTS)
        // Handle errors then break stream
        .for_each(|b| async move {
            if let Err(err) = b {
                println!("Error processing {:?}", err);
            }
        })
        .await;
    println!("chunks: {}, bytes:{}", total_chunks, bytes_size);
    Ok(())
}
