#![allow(unused_imports)]
use core::str;
use std::{io::{Read, Write}, net::TcpListener};


fn main() {
    let host = "127.0.0.1:6379";
    println!("Redis server started at host: {host}");
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind(host).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = [0; 512];
                stream.read(&mut buf).unwrap();

                println!("Request: {}", String::from_utf8_lossy(&buf[..]));
                stream.write(b"+PONG\r\n").unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
