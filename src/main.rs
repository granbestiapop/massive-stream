use futures::stream::{StreamExt, TryStreamExt};
use reqwest::Client;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::compat::FuturesAsyncReadCompatExt;

const CONCURRENT_REQUESTS: usize = 100;
const S3_FILE_URL: &str = "http://localhost:8080/a/test";
const UPLOADER_DATA_PATH: &str = "http://localhost:8080/a/";

const LOCAL_FILE_DIR: &str = "data/foo.txt";

async fn do_call(client: &Client, url: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}{}", UPLOADER_DATA_PATH, url);
    let result = client.get(&url).send().await?.text().await?;
    //tokio::time::delay_for(tokio::time::Duration::from_millis(1000)).await;
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let file = File::open(LOCAL_FILE_DIR).await?;
    let client = Client::new();

    let chunk_size: usize = 1024 * 1024; // 1M
    let mut total_chunks: u32 = 0;
    let mut bytes_size: usize = 0;

    let response = client.get(S3_FILE_URL).send().await?;
    let http_stream = response
        .bytes_stream()
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read()
        .compat();

    let file_stream = BufReader::new(file);

    let cursor = http_stream
        .lines()
        .map(Result::unwrap)
        .map(|line| {
            let size = line.bytes().len();
            bytes_size += size;
            if bytes_size >= chunk_size {
                bytes_size -= chunk_size;
                total_chunks += 1;
            }
            println!("{}", line);
            //println!("chunks: {}, bytes:{}", total_chunks, bytes_size);
            line
        })
        .map(|st| do_call(&client, st))
        .buffer_unordered(CONCURRENT_REQUESTS);

    cursor
        .for_each(|b| async move {
            if let Err(err) = b {
                println!("Error processing {:?}", err);
                //Ok(a) => println!("Finish {}", a),
                //Err(err) => println!("Error {:?}", err),
            }
        })
        .await;
    println!("chunks: {}, bytes:{}", total_chunks, bytes_size);
    Ok(())
}
