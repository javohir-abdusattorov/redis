#![allow(unused_imports)]
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;
use resp::RespHandler;
use db::Database;

mod resp;
mod operation;
mod metadata;
mod parser;
mod serializer;
mod db;

#[tokio::main]
async fn main() {
    let host = "127.0.0.1:6378";
    let listener = TcpListener::bind(host).await.unwrap();
    println!("Redis server started at host: {host}");

    let db = Arc::new(Mutex::new(Database::new()));

    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((stream, addr)) => {
                let db = Arc::clone(&db);
                println!("connection: {addr:?}");

                tokio::spawn(async move {
                    RespHandler::new(
                        stream,
                        db,
                    )
                    .process().await
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}