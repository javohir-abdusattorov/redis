use std::{fs::File, io::Read};
use super::operation::Operation;


impl Operation {
    pub fn to_bytes(&self) -> Vec<u8> {
        let crlf = "\r\n";
        match self {
            Operation::String(str) => format!("+{str}{crlf}").as_bytes().to_vec(),
            Operation::Bulk(str) => format!("${}{crlf}{str}{crlf}", str.len()).as_bytes().to_vec(),
            Operation::Integer(num) => format!(":{num}{crlf}").as_bytes().to_vec(),
            Operation::Error(str) => format!("-{str}{crlf}").as_bytes().to_vec(),
            Operation::Null() => format!("$-1{crlf}").as_bytes().to_vec(),
            Operation::Array(vec) | Operation::Sequential(vec) => {
                let array = vec
                    .into_iter()
                    .map(|op| op.to_bytes())
                    .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                    .collect::<String>();

                    format!("*{}{crlf}{array}", vec.len()).as_bytes().to_vec()
            },
            Operation::File(path) => {
                let mut file = File::open(path).unwrap();
                let mut contents = Vec::new();
                let length = file.read_to_end(&mut contents).unwrap();
                let mut header = format!("${length}{crlf}")
                    .as_bytes()
                    .to_vec();

                header.extend(contents);
                header
            },
        }
    }
}