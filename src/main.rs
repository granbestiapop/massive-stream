use futures::stream::{StreamExt, TryStreamExt};
use tokio::io::{AsyncBufReadExt};
use tokio_util::compat::FuturesAsyncReadCompatExt;

const CONCURRENT_REQUESTS: usize = 100;
const S3_FILE_URL: &str = "http://localhost:8080/bigfile.txt";
const UPLOADER_DATA_PATH: &str = "http://localhost:8080/topic/";

mod repository;
use repository::client::RestClient;

async fn do_call(client: &RestClient, path: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}{}", UPLOADER_DATA_PATH, path);
    println!("Starting {}", url);
    let req = client
        .request(reqwest::Method::POST, url.as_str())
        .build()?;
    let text = client.execute(req).await?.text().await?;
    Ok(text)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    //let file = File::open(LOCAL_FILE_DIR).await?;
    //let file_stream = BufReader::new(file);
    let chunk_size: usize = 1024 * 1024; // 1M
    let mut total_chunks: u32 = 0;
    let mut bytes_size: usize = 0;

    let restclient = RestClient::new()?;

    let req = restclient
        .request(reqwest::Method::GET, S3_FILE_URL)
        .build()?;
    let response =  restclient.stream(req).await?;
    response
        //# Input Stream
        .bytes_stream()
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read()
        .compat()
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
        .map(|st| do_call(&restclient, st))
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
