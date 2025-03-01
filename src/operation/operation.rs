use anyhow::Result;


#[derive(Debug, Clone)]
pub enum Operation {
    String(String),
    Bulk(String),
    Array(Vec<Operation>),
    Integer(i128),
    Error(String),
    Null()
}

impl Operation {
    pub fn only_array(self) -> Result<(String, Vec<Operation>)> {
        match self {
            Operation::Array(vec) => Ok((
                vec.first().unwrap().clone().only_bulk()?,
                vec.into_iter().skip(1).collect(),
            )),
            _ => Err(anyhow::anyhow!("Unexpected command to only_array: {self:?}")),
        }
    }

    pub fn only_bulk(self) -> Result<String> {
        match self {
            Operation::Bulk(str) => Ok(str),
            _ => Err(anyhow::anyhow!("Unexpected command to only_bulk: {self:?}"))
        }
    }
}