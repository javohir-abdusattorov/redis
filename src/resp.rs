use bytes::BytesMut;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};
use anyhow::{Ok, Result};

#[derive(Debug, Clone)]
pub enum Operation {
    String(String),
    Bulk(String),
    Array(Vec<Operation>),
    Error(String),
}

impl Operation {
    pub fn parse(buffer: BytesMut) -> Result<(Operation, usize)> {
        // println!("raw: {buffer:?}");
        match buffer[0] as char {
            '+' => parse_string(buffer),
            '$' => parse_bulk(buffer),
            '*' => parse_array(buffer),
            _ => Err(anyhow::anyhow!("Not a known operation type: {:?}", buffer)),
        }
    }

    pub fn serialiaze(self) -> String {
        let crlf = "\r\n";
        match self {
            Operation::String(str) => format!("+{str}{crlf}"),
            Operation::Bulk(str) => format!("${}{crlf}{str}{crlf}", str.len()),
            Operation::Array(vec) => format!(
                "*{}{}",
                vec.len(),
                vec.into_iter().map(|op| op.serialiaze()).collect::<String>()
            ),
            Operation::Error(str) => format!("-{str}{crlf}")
        }
    }
}

pub struct RespHandler {
    stream: TcpStream,
    buffer: BytesMut,

}

impl RespHandler {
    pub fn new(stream: TcpStream) -> Self {
        RespHandler {
            stream: stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<Operation>> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;
        if bytes_read == 0 {
            return Ok(None);
        }

        let (operation, _) = Operation::parse(self.buffer.split())?;
        Ok(Some(operation))
    }

    pub async fn write_value(&mut self, operation: Operation) -> Result<()> {
        println!("response: {:?}", operation);
        self.stream.write(operation.serialiaze().as_bytes()).await?;
        Ok(())
    }
}

fn parse_string(buffer: BytesMut) -> Result<(Operation, usize)> {
    if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let string = String::from_utf8(line.to_vec()).unwrap();

        return Ok((
            Operation::String(string),
            len + 1,
        ));
    }

    Err(anyhow::anyhow!("Invalid string: {:?}", buffer))
}

fn parse_array(buffer: BytesMut) -> Result<(Operation, usize)> {
    let (array_len, mut bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let array_len = parse_int(line)?;

        (array_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format: {:?}", buffer));
    };

    let mut operations = Vec::new();
    for _ in 0..array_len {
        let (array_item, len) = Operation::parse(BytesMut::from(&buffer[bytes_consumed..]))?;

        bytes_consumed += len;
        operations.push(array_item);
    }

    Ok((
        Operation::Array(operations),
        bytes_consumed,
    ))
}

fn parse_bulk(buffer: BytesMut) -> Result<(Operation, usize)> {
    let (bulk_len, bytes_consumed) = if let Some((line, len)) = read_until_crlf(&buffer[1..]) {
        let bulk_len = parse_int(line)?;

        (bulk_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format: {:?}", buffer));
    };

    let end_of_bulk = bytes_consumed + bulk_len as usize;

    Ok((
        Operation::Bulk(String::from_utf8(buffer[bytes_consumed..end_of_bulk].to_vec())?),
        end_of_bulk + 2,
    ))
}

fn parse_int(buffer: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i -1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }

    return None;
}