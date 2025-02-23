use std::sync::{Arc, Mutex};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use bytes::BytesMut;
use anyhow::Result;
use crate::{db::Database, metadata::Metadata, operation::Operation, command::Command};


pub struct RespHandler{
    stream: TcpStream,
    buffer: BytesMut,
    db: Arc<Mutex<Database>>,
}

impl RespHandler {
    pub fn new(stream: TcpStream, db: Arc<Mutex<Database>>) -> RespHandler{
        RespHandler {
            stream: stream,
            buffer: BytesMut::with_capacity(512),
            db: db,
        }
    }

    pub async fn process(&mut self) {
        loop {
            let value: Result<Option<Operation>> = self.read_value().await;
            let unexpected_err = |err| anyhow::anyhow!(format!("[Handler] Unexpected request: {err:?}"));

            let error_handle = || -> Result<Option<Operation>> {
                match value.map_err(unexpected_err)? {
                    None => Ok(None),
                    Some(operation) => {
                        let command = Command::try_from(operation)?;
                        let result = self.route(command)?;

                        Ok(Some(result))
                    },
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

    fn route(&self, command: Command) -> Result<Operation> {
        match command.as_str() {
            "ping" | "command" => Ok(Operation::String("PONG".to_string())),
            "echo" => Ok(Operation::Bulk(command.single_argument()?)),
            "get" => {
                let key = command.single_argument()?;
                match self.db.lock().unwrap().get(&key) {
                    None => Ok(Operation::Null()),
                    Some(result) => Ok(Operation::Bulk(result)),
                }
            },
            "set" => {
                let (key, value) = command.first_2_arguments()?;

                let metadata_parameters = command.optional_arguments_after(2);
                let metadata = Metadata::try_from(metadata_parameters)?;

                self.db.lock().unwrap().set(&key, value, metadata);
                Ok(Operation::String("OK".to_string()))
            },
            "expire" => {
                let (key, expire_seconds) = command.first_2_arguments()?;
                let seconds = expire_seconds.parse::<u128>().map_err(|_| anyhow::anyhow!(format!("Expire command invalid arguments: cannot parse to integer")))?;

                let metadata = Metadata::try_from(seconds).unwrap();
                let timestamp = self.db.lock().unwrap().set_expire(&key, metadata).unwrap_or(0);

                Ok(Operation::Integer(timestamp as i128))
            },
            "del" => {
                let key = command.single_argument()?;
                self.db.lock().unwrap().del(&key);
                Ok(Operation::Integer(1))
            },
            "ttl" => {
                let key = command.single_argument()?;
                let ttl = self.db.lock().unwrap().ttl(&key);
                Ok(Operation::Integer(ttl))
            },
            "keys" => {
                let pattern = command.single_argument()?;
                let result = self.db.lock().unwrap()
                    .search(&pattern)
                    .into_iter()
                    .filter(|value| self.db.lock().unwrap().try_expire(value).is_none())
                    .map(|value| Operation::Bulk(value))
                    .collect();

                    Ok(Operation::Array(result))
            },
            unknown_command => Err(anyhow::anyhow!(format!("[Handler] Unexpected command: {unknown_command:?}")))
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
        // println!("response: {:?}", operation.clone().to_string());
        self.stream.write(operation.to_string().as_bytes()).await?;
        Ok(())
    }
}