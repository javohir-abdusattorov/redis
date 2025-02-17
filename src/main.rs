#![allow(unused_imports)]
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;

mod resp;
use resp::*;

#[tokio::main]
async fn main() {
    let host = "127.0.0.1:6379";
    println!("Redis server started at host: {host}");

    let listener = TcpListener::bind(host).await.unwrap();
    loop {
        let stream = listener.accept().await;

        match stream {
            Ok((stream, addr)) => {
                println!("connection: {addr:?}");
                tokio::spawn(async move {
                    handle_connection(stream).await
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_connection(stream: TcpStream) {
    let mut handler = RespHandler::new(stream);

    loop {
        let value = handler.read_value().await.unwrap();

        let response = if let Some(value) = value {
            println!("command: {value:?}");
            let (command, args) = extract_command(value).unwrap();
            match command.to_lowercase().as_str() {
                "ping" | "command" => Operation::String("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                command => Operation::Error(format!("Unexpected command: {command:?}"))
            }
        } else {
            break;
        };

        handler.write_value(response).await.unwrap();
    }
}

fn extract_command(operation: Operation) -> Result<(String, Vec<Operation>)> {
    match operation {
        Operation::Array(vec) => Ok((
            unpack_bulk(vec.first().unwrap().clone())?,
            vec.into_iter().skip(1).collect(),
        )),
        _ => Err(anyhow::anyhow!("Unexpected command format: {operation:?}")),
    }
}

fn unpack_bulk(operation: Operation) -> Result<String> {
    match operation {
        Operation::Bulk(str) => Ok(str),
        _ => Err(anyhow::anyhow!("Unexpected bulk string format: {operation:?}"))
    }
}