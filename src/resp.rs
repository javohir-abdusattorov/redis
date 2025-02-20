use std::sync::{Arc, Mutex};
use itertools::Itertools;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use bytes::BytesMut;
use anyhow::Result;
use crate::{db::Database, metadata::Metadata, operation::Operation};


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
            let response = match self.read_value().await {
                Err(err) => Operation::Error(format!("[Handler] Unexpected error: {err:?}")),
                Ok(command) => {
                    if let Some(operation) = command {
                        // println!("command: {operation:?}");
                        let (command, args) = operation.only_array().unwrap();
                        self.route(command, args)
                    }
                    else {
                        break;
                    }
                }
            };

            self.write_value(response).await.unwrap();
        }
    }

    fn route(&self, command: String, args: Vec<Operation>) -> Operation {
        match command.to_lowercase().as_str() {
            "ping" | "command" => Operation::String("PONG".to_string()),
            "echo" => args.first().unwrap().clone(),
            "set" => {
                let mut parameters = [None, None, None, None];

                args
                    .iter()
                    .map(|arg| arg.clone().only_bulk().unwrap())
                    .enumerate()
                    .for_each(|(i, arg)| parameters[i] = Some(arg));

                let (key, value) = match (&parameters[0], &parameters[1]) {
                    (Some(key), Some(value)) => (key.clone(), value.clone()),
                    _ => return Operation::Error(format!("Set command invalid arguments: {:?}", args.iter().map(|arg| arg.to_string()).join(", "))),
                };

                let metadata_parameters = parameters
                    .into_iter()
                    .skip(2)
                    .filter_map(|arg| arg)
                    .collect::<Vec<String>>();
                let metadata = match Metadata::try_from(metadata_parameters) {
                    Ok(metadata) => metadata,
                    Err(err) => return Operation::Error(err.to_string()),
                };

                self.db.lock().unwrap().set(&key, value, metadata);
                Operation::String("OK".to_string())
            },
            "get" => {
                let key = args.first().unwrap().clone().only_bulk().unwrap();
                match self.db.lock().unwrap().get(&key) {
                    None => Operation::Null(),
                    Some(result) => Operation::Bulk(result)
                }
            },
            any_command => Operation::Error(format!("Unexpected command: {any_command:?}"))
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