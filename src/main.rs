#![allow(unused_imports)]
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;
use resp::RespHandler;

mod resp;
mod operation;
mod parser;
mod serializer;

#[tokio::main]
async fn main() {
    let host = "127.0.0.1:6378";
    println!("Redis server started at host: {host}");

    let listener = TcpListener::bind(host).await.unwrap();
    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((stream, addr)) => {
                println!("connection: {addr:?}");
                tokio::spawn(async move { RespHandler::new(stream).process().await });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}