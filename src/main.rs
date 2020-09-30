use futures::stream::{StreamExt, TryStreamExt};
use tokio::io::AsyncBufReadExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;

#[macro_use]
extern crate lazy_static;

mod repository;
use repository::client::RestClient;

const CONCURRENT_REQUESTS: usize = 100;
const DEFAULT_FILE_URL: &str = "http://localhost:8080/stream";
const DEFAULT_TARGET: &str = "http://localhost:8080/topic";

lazy_static! {
    static ref FILE_URL: String = std::env::var("FILE").unwrap_or(DEFAULT_FILE_URL.to_string());
    static ref TARGET: String = std::env::var("TARGET").unwrap_or(DEFAULT_TARGET.to_string());
}

async fn do_call(client: &RestClient, body: String) -> Result<String, Box<dyn std::error::Error>> {
    let req = client
        .request(reqwest::Method::POST, TARGET.as_str())
        .body(body)
        .build()?;
    let text = client.execute(req).await?.text().await?;
    Ok(text)
}

fn check_error(next_range: u64, result: &Result<String, std::io::Error>) -> bool {
    match result {
        Ok(_) => return true,
        Err(e) => {
            println!(
                "Error on stream! {:?} should retry from bytes {}",
                e,
                next_range + 1
            );
            false
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let chunk_size: usize = 1024 * 1024; // 1M
    let mut total_chunks: u32 = 0;
    let mut bytes_size: usize = 0;
    let mut next_range: u64 = 0;

    let restclient = RestClient::new()?;

    let file_size = file_size(&restclient).await?;

    let req = restclient
        .request(reqwest::Method::GET, FILE_URL.as_str())
        .build()?;
    let response = restclient.stream(req).await?;

    // INPUT stream with lines
    let line_stream = response
        .bytes_stream()
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read()
        .compat()
        .lines();

    // Hack double trait
    let stream = tokio::stream::StreamExt::map(line_stream, |a| a);
    let stream =
        tokio::stream::StreamExt::take_while(stream, |result| check_error(next_range, result));
    //# Transformer Stream
    stream
        .map(Result::unwrap)
        .map(move |line| {
            let size = line.bytes().len();
            bytes_size += size;
            next_range += size as u64;
            if bytes_size >= chunk_size {
                bytes_size -= chunk_size;
                total_chunks += 1;
            }
            //println!("chunks: {}, bytes:{}", total_chunks, bytes_size);
            //println!("line {}", line);
            line
        })
        //# Output Stream
        .map(|st| do_call(&restclient, st))
        .buffer_unordered(CONCURRENT_REQUESTS)
        // Handle errors then break stream
        .for_each(|b| async move {
            if let Err(err) = b {
                println!("Error processing {:?}", err);
            }
        })
        .await;

    if next_range < file_size {
        // On this step should retry cause don't complete stream
        println!("next range {} file size{}", next_range, file_size);
    }
    println!("chunks: {}, bytes:{}", total_chunks, bytes_size);
    Ok(())
}

async fn file_size(restclient: &RestClient) -> Result<u64, reqwest::Error> {
    let head_request = restclient
        .request(reqwest::Method::HEAD, FILE_URL.as_str())
        .build()?;
    let head_response = restclient.execute(head_request).await?;
    Ok(head_response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH.as_str())
        .expect("On HEAD don't retrieve content-length header")
        .to_str()
        .unwrap()
        .parse::<u64>()
        .expect("Invalid content-length header"))
}


async fn process(){
    
}