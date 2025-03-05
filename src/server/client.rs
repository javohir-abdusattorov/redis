use std::{io::{Read, Write}, net::TcpStream};
use anyhow::Result;
use bytes::BytesMut;
use itertools::Itertools;
use crate::operation::operation::Operation;

pub struct Client {
    stream: TcpStream,
    read_buffer: [u8; 512],
}

impl Client {
    pub fn connect(address: String) -> Result<Self> {
        Ok(Client {
            stream: TcpStream::connect(address)?,
            read_buffer: [0; 512],
        })
    }

    pub fn send(&mut self, command: Vec<String>) -> Result<Operation> {
        let command = command.into_iter().map(|s| Operation::Bulk(s)).collect();
        self.stream.write_all(&Operation::Array(command).to_bytes())?;

        let read_bytes = self.stream.read(&mut self.read_buffer)?;
        let operation = Operation::try_from(BytesMut::from(&self.read_buffer[..read_bytes]).split())?;
        Ok(operation)
    }

    pub fn receive_file(&mut self) -> Result<Vec<u8>> {
        let mut file_buf = Vec::new();

        loop {
            let read_bytes = self.stream.read(&mut self.read_buffer)?;
            file_buf.extend_from_slice(&self.read_buffer[..read_bytes]);
            if read_bytes < 512 { break; }
        }

        let end_of_len = file_buf.iter().find_position(|c| **c as char == '\n').unwrap();
        let buf = file_buf.split_off(end_of_len.0 + 1);

        Ok(buf)
    }
}