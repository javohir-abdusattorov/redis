use std::sync::{Arc, Mutex};
use anyhow::{Ok, Result};
use super::command::Command;
use crate::config::Config;
use crate::resp::operation::Operation;
use crate::{db::Database, metadata::Metadata};


pub struct Router {
    config: Arc<Config>,
    db: Arc<Mutex<Database>>,
}

impl Router {
    pub fn new(config: Arc<Config>, db: Arc<Mutex<Database>>) -> Self {
        Router {
            config,
            db,
        }
    }

    pub fn handle(&mut self, operation: Operation) -> Result<Operation> {
        let command = Command::try_from(operation)?;
        match command.as_str() {
            "ping" | "command" => self.pong(),
            "echo" => self.echo(command),
            "get" => self.get(command),
            "set" => self.set(command),
            "expire" => self.expire(command),
            "del" => self.del(command),
            "ttl" => self.ttl(command),
            "keys" => self.keys(command),
            "config" => self.config(command),
            unknown_command => Err(anyhow::anyhow!(format!("[Router] Unexpected command: {unknown_command:?}")))
        }
    }

    fn pong(&self) -> Result<Operation> {
        Ok(Operation::String("PONG".to_string()))
    }

    fn echo(&self, command: Command) -> Result<Operation> {
        Ok(Operation::Bulk(command.single_argument()?))
    }

    fn get(&self, command: Command) -> Result<Operation> {
        let key = command.single_argument()?;
        match self.db.lock().unwrap().get(&key) {
            None => Ok(Operation::Null()),
            Some(result) => Ok(Operation::Bulk(result)),
        }
    }

    fn set(&self, command: Command) -> Result<Operation> {
        let (key, value) = command.first_2_arguments()?;

        let metadata_parameters = command.optional_arguments_after(2);
        let metadata = Metadata::try_from(metadata_parameters)?;

        self.db.lock().unwrap().set(&key, value, metadata);
        Ok(Operation::String("OK".to_string()))
    }

    fn expire(&self, command: Command) -> Result<Operation> {
        let (key, expire_seconds) = command.first_2_arguments()?;
        let seconds = expire_seconds.parse::<u128>().map_err(|_| anyhow::anyhow!(format!("Expire command invalid arguments: cannot parse to integer")))?;

        let metadata = Metadata::try_from(seconds).unwrap();
        let timestamp = self.db.lock().unwrap().set_expire(&key, metadata).unwrap_or(0);

        Ok(Operation::Integer(timestamp as i128))
    }

    fn del(&self, command: Command) -> Result<Operation> {
        let key = command.single_argument()?;
        self.db.lock().unwrap().del(&key);
        Ok(Operation::Integer(1))
    }

    fn ttl(&self, command: Command) -> Result<Operation> {
        let key = command.single_argument()?;
        let ttl = self.db.lock().unwrap().ttl(&key);
        Ok(Operation::Integer(ttl))
    }

    fn keys(&self, command: Command) -> Result<Operation> {
        let pattern = command.single_argument()?;
        let result = self.db.lock().unwrap().search(&pattern);

        // This can be done without re-defining variable, but db mutex lock would not have been dropped
        // we need db mutex lock inside mapping / filtering of search result
        let result = result.into_iter()
            .filter(|value| self.db.lock().unwrap().try_expire(value).is_none())
            .map(|value| Operation::Bulk(value))
            .collect();

        Ok(Operation::Array(result))
    }

    fn config(&self, command: Command) -> Result<Operation> {
        let (command, config) = command.first_2_arguments()?;
        match command.as_str() {
            "get" => {
                let result = match self.config.get_by_key(&config) {
                    None => vec![],
                    Some(value) => vec![config, value.clone()],
                };

                let result = result.into_iter()
                    .map(|value| Operation::Bulk(value))
                    .collect();

                Ok(Operation::Array(result))
            },
            any_command => Err(anyhow::anyhow!("Unexpected CONFIG command: {any_command}"))
        }
    }
}