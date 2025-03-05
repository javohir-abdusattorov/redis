use std::sync::{Arc, Mutex};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use bytes::BytesMut;
use anyhow::Result;
use super::router::Router;
use crate::{operation::operation::Operation, replication::replicator::Replicator};


pub struct Handler {
    stream: TcpStream,
    buffer: BytesMut,
    router: Router,
}

impl Handler {
    pub fn new(stream: TcpStream, router: Router) -> Self {
        Handler {
            stream,
            router,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn process(&mut self) {
        loop {
            let value: Result<Option<Operation>> = self.read_value().await;
            let unexpected_err = |err| anyhow::anyhow!(format!("[Handler] Unexpected request: {err:?}"));

            let error_handle = || -> Result<Option<Operation>> {
                match value.map_err(unexpected_err)? {
                    None => Ok(None),
                    Some(operation) => Ok(Some(self.router.handle(operation)?)),
                }
            };

            let response = match error_handle() {
                Err(err) => Operation::Error(err.to_string()),
                Ok(response) => match response {
                    None => break,
                    Some(value) => value,
                },
            };

            self.write_value(response).await.unwrap()
        }
    }

    async fn read_value(&mut self) -> Result<Option<Operation>> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;
        if bytes_read == 0 {
            return Ok(None);
        }

        Ok(Some(Operation::try_from(self.buffer.split())?))
    }

    async fn write_value(&mut self, operation: Operation) -> Result<()> {
        match operation {
            Operation::Sequential(sequence) => {
                for operation in sequence {
                    self.stream.write(&operation.to_bytes()).await?;
                }
            },
            _ => {
                self.stream.write(&operation.to_bytes()).await?;
            }
        };

        Ok(())
    }
}