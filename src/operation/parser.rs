use bytes::BytesMut;
use anyhow::Result;
use super::operation::Operation;


impl TryFrom<BytesMut> for Operation {
    type Error = anyhow::Error;

    fn try_from(value: BytesMut) -> Result<Self> {
        Ok(Parser::new(value).parse()?.0)
    }
}

pub struct Parser {
    indicator: char,
    buffer: BytesMut,
}

impl Parser {
    pub fn new(buffer: BytesMut) -> Parser {
        Parser {
            indicator: buffer[0] as char,
            buffer,
        }
    }

    pub fn parse(&self) -> Result<(Operation, usize)> {
        match self.indicator {
            '+' => self.from_string(),
            '$' => self.from_bulk(),
            '*' => self.from_array(),
            _ => Err(anyhow::anyhow!("[Parser] Not a known operation type: {:?}", self.buffer)),
        }
    }

    fn from_string(&self) -> Result<(Operation, usize)> {
        if let Some((line, len)) = self.read_until_crlf(&self.buffer[1..]) {
            let string = String::from_utf8(line.to_vec()).unwrap();
    
            return Ok((
                Operation::String(string),
                len + 1,
            ));
        }

        Err(anyhow::anyhow!("[Parser] Invalid string: {:?}", self.buffer))
    }

    fn from_bulk(&self) -> Result<(Operation, usize)> {
        let (bulk_len, bytes_consumed) = if let Some((line, len)) = self.read_until_crlf(&self.buffer[1..]) {
            let bulk_len = self.parse_int(line)?;
    
            (bulk_len, len + 1)
        } else {
            return Err(anyhow::anyhow!("[Parser] Invalid bulk format: {:?}", self.buffer));
        };

        let end_of_bulk = bytes_consumed + bulk_len as usize;
    
        Ok((
            Operation::Bulk(String::from_utf8(self.buffer[bytes_consumed..end_of_bulk].to_vec())?),
            end_of_bulk + 2,
        ))
    }

    fn from_array(&self) -> Result<(Operation, usize)> {
        let (array_len, mut bytes_consumed) = if let Some((line, len)) = self.read_until_crlf(&self.buffer[1..]) {
            let array_len = self.parse_int(line)?;
    
            (array_len, len + 1)
        } else {
            return Err(anyhow::anyhow!("[Parser] Invalid array format: {:?}", self.buffer));
        };
    
        let mut operations = Vec::new();
        for _ in 0..array_len {
            let (array_item, len) = Parser::new(BytesMut::from(&self.buffer[bytes_consumed..])).parse()?;
    
            bytes_consumed += len;
            operations.push(array_item);
        }
    
        Ok((
            Operation::Array(operations),
            bytes_consumed,
        ))
    }

    fn parse_int(&self, buffer: &[u8]) -> Result<i64> {
        Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
    }

    fn read_until_crlf<'a>(&self, buffer: &'a [u8]) -> Option<(&'a [u8], usize)> {
        for i in 1..buffer.len() {
            if buffer[i -1] == b'\r' && buffer[i] == b'\n' {
                return Some((&buffer[0..(i - 1)], i + 1));
            }
        }

        None
    }
}