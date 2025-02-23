use crate::operation::Operation;


impl ToString for Operation {
    fn to_string(&self) -> String {
        let crlf = "\r\n";
        match self {
            Operation::String(str) => format!("+{str}{crlf}"),
            Operation::Bulk(str) => format!("${}{crlf}{str}{crlf}", str.len()),
            Operation::Array(vec) => format!(
                "*{}{crlf}{}",
                vec.len(),
                vec.into_iter().map(|op| op.to_string()).collect::<String>()
            ),
            Operation::Integer(num) => format!(":{num}{crlf}"),
            Operation::Error(str) => format!("-{str}{crlf}"),
            Operation::Null() => format!("$-1{crlf}"),
        }
    }
}