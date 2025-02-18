use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};
use bytes::BytesMut;
use anyhow::Result;
use crate::operation::Operation;


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

    pub async fn process(&mut self) {
        loop {
            let response = match self.read_value().await {
                Err(err) => Operation::Error(format!("[Handler] Unexpected error: {err:?}")),
                Ok(command) => {
                    if let Some(operation) = command {
                        println!("command: {operation:?}");
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
        println!("response: {:?}", operation);
        self.stream.write(operation.to_string().as_bytes()).await?;
        Ok(())
    }
}