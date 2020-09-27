use futures::future;
use futures::stream::{StreamExt, TryStreamExt};
use reqwest::Client;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
//use tokio::stream::{StreamExt};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio_util::codec::{Framed, LinesCodec};
use tokio_util::compat::FuturesAsyncReadCompatExt;

const CONCURRENT_REQUESTS: usize = 100;

async fn do_call(client: &Client, url: String) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("http://localhost:8080/a/{}", url);
    let result = client.get(&url).send().await?.text().await?;
    //tokio::time::delay_for(tokio::time::Duration::from_millis(1000)).await;

    //let result = String::from("");
    Ok(result)
}

#[tokio::main]
async fn main() {
    let file = File::open("data/foo.txt").await.unwrap();
    let client = Client::new();

    let chunk_size: usize = 1024 * 1024; // 1M
    let mut total_chunks: u32 = 0;
    let mut bytes_size: usize = 0;

    let response = client
        .get("http://localhost:8080/a/test")
        .send()
        .await
        .unwrap();
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
                println!("Error {:?}", err);
                //Ok(a) => println!("Finish {}", a),
                //Err(err) => println!("Error {:?}", err),
            }
        })
        .await;
    println!("chunks: {}, bytes:{}", total_chunks, bytes_size);
}

/*
let url = "127.0.0.1:8080"; //"raw.githubusercontent.com:843";
let mut sock_stream = TcpStream::connect(url).await.unwrap();
sock_stream
    .write_all(b"GET /a/test HTTP/1.1\r\n\r\n") // /granbestiapop/json-mocks/master/db.json
    .await
    .unwrap();

let frame = Framed::new(sock_stream, LinesCodec::new());
frame
    .map(Result::unwrap)
    .skip_while(|s| future::ready(!s.is_empty()))
    .skip_while(|s| future::ready(s.is_empty()))
    .for_each(|a| async move {
        println!("Data from socket {:?}", a);
    })
    .await;*/
