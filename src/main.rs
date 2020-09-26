use futures::stream::StreamExt;
use futures::future;
use reqwest::Client;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
//use tokio::stream::{StreamExt};
use tokio_util::codec::LinesCodec;
use tokio_util::codec::Framed;
use tokio::net::TcpStream;
use tokio::prelude::*;

//use tokio::util;

const CONCURRENT_REQUESTS: usize = 10;


async fn do_call(client: &Client, url: String) -> Result<String, Box<std::error::Error>> {
    let url = format!("http://localhost:8080/a/{}", url);
    let result = client.get(&url).send().await?.text().await?;
    //tokio::time::delay_for(Duration::from_millis(1000)).await;
    Ok(result)
}

#[tokio::main]
async fn main() {
    let file = File::open("data/foo.txt").await.unwrap();
    let client = Client::new();


    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    stream.write_all(b"GET /a/test HTTP/1.1\r\n\r\n").await.unwrap();

    
    let frame = Framed::new(stream, LinesCodec::new());
    frame.map(Result::unwrap)
        .skip_while(|s|future::ready(!s.is_empty()))
        .skip_while(|s|future::ready(s.is_empty()))        
        .for_each(|a| async move{
            println!("lalalal {:?}", a);
        }).await;
        

    /*
    let mut stream_http = reqwest::get("http://httpbin.org/ip").await
        .unwrap()
        .bytes_stream();*/

    //let frame = Framed::new(stream, LinesCodec::new());

        //.split("/n");
        

    let cursor = BufReader::new(file)
        .lines()
        .map(Result::unwrap)
        .map(|st|do_call(&client, st))
        .buffer_unordered(CONCURRENT_REQUESTS);

    cursor
        .for_each(|b| async move {
            match b {
                Ok(a) =>  println!("Finish {}", a),
                Err(err) => println!("Error {:?}", err),
            }
        })
        .await;
}
