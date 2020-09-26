use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
//use tokio::stream::StreamExt;
use futures::stream::StreamExt;
//use tokio::sync::{mpsc, oneshot};
use reqwest::Client;

#[tokio::main]
async fn main() {
    //let mut rt = runtime::Builder::new().core_threads(4).build().unwrap();

    //let (mut tx, mut rx) = mpsc::channel::<Vec<String>>(1);
    //let (to, ro) = oneshot::channel::<i32>();

    let file = File::open("foo.txt").await.unwrap();

    let client = Client::new();

    let cursor = BufReader::new(file)
        .lines()
        .map(|st| {
            let st = st.unwrap().clone();
            println!("runnning {}", st);
            let client = &client;
            async move {
                let url = format!("http://localhost:8080/a/{}", st);
                let resp = client.get(&url).send().await.unwrap().text().await.unwrap();
                //tokio::time::delay_for(Duration::from_millis(1000)).await;
                resp
            }

        })
        .buffer_unordered(2);

    cursor
        .for_each(|b| async move {
            println!("finish {}", b);
        })
        .await;
    //let mut buffer = String::new();
    //reader.read_line(&mut buffer).await;
    //file.read_to_string(&mut contents).await?;

    //let mut lines = reader.lines();

    /*
    task::spawn(async move {
        if let Some(s) = rx.recv().await {
            println!("{}", s);
            tokio::time::delay_for(Duration::from_millis(1000)).await;
            println!("finish {}", s);
        }
    });*/

    /*
    tokio::spawn(async move {
        while let Some(line) = lines.next_line().await.unwrap() {
            tx.send(vec![line]).await;
        }
        to.send(1).unwrap();
    });

    loop {
        while let Some(s) = rx.recv().await {
            println!("{:?}", s);
            tokio::time::delay_for(Duration::from_millis(1000)).await;
            println!("finish {:?}", s);
        }
    }

    ro.await.unwrap();*/

    //println!("len = {}", buffer.len());
    //println!("len = {:?}", buffer);

    //Ok(())
}

/*
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut file = File::open("foo.txt").await?;

    let mut segments = file.split(b'f');

while let Some(segment) = segments.next_segment().await? {
    println!("length = {}", segment.len())
}

    Ok(())
}*/
