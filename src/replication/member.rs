use super::role::ReplicationRole;
use std::net::TcpStream;
use anyhow::{anyhow, Result};
use itertools::Itertools;


#[derive(Debug)]
pub struct ReplicationMember {
    pub id: String,
    pub role: ReplicationRole,
    pub address: String,
    stream: Option<TcpStream>,
}

impl ReplicationMember {
    pub fn new(role: ReplicationRole, id: String, address: String) -> Self {
        ReplicationMember {
            id,
            role,
            address,
            stream: None,
        }
    }

    pub fn connect(&mut self) -> Result<&mut Option<TcpStream>> {
        if let None = self.stream {
            self.stream = Some(TcpStream::connect(self.address.clone())?);
        }

        Ok(&mut self.stream)
    }
}