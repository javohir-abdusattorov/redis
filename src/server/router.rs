use super::{command::Command, constants::{commands, responses}};
use crate::{config::Config, replication::replicator::Replicator};
use crate::operation::metadata::Metadata;
use crate::operation::operation::Operation;
use crate::storage::db::Database;
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub struct Router {
    config: Arc<Config>,
    db: Arc<Mutex<Database>>,
    replicator: Arc<Mutex<Replicator>>,
}

impl Router {
    pub fn new(config: Arc<Config>, db: Arc<Mutex<Database>>, replicator: Arc<Mutex<Replicator>>) -> Self {
        Router { config, db, replicator }
    }

    pub fn handle(&mut self, operation: Operation) -> Result<Operation> {
        let command = Command::try_from(operation.clone())?;
        let is_write = command.is_write();
        println!("[Command] {}", &command.can_match());

        let result = match command.can_match().as_str() {
            commands::PING | commands::COMMAND => self.pong(),
            commands::ECHO => self.echo(command),
            commands::GET => self.get(command),
            commands::SET => self.set(command),
            commands::EXPIRE => self.expire(command),
            commands::DEL => self.del(command),
            commands::TTL => self.ttl(command),
            commands::KEYS => self.keys(command),
            commands::CONFIG => self.config(command),
            commands::INFO => self.info(command),
            commands::REPLCONF => self.replconf(command),
            commands::PSYNC => self.psync(command),
            unknown_command => Err(anyhow::anyhow!(format!(
                "[Router] Unexpected command: {unknown_command:?}"
            ))),
        }?;

        if is_write {
            self.replicator.lock().unwrap().distribute(operation);
        }

        Ok(result)
    }

    fn pong(&self) -> Result<Operation> {
        Ok(Operation::String(responses::PONG.to_string()))
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
        let seconds = expire_seconds.parse::<u128>().map_err(|_| {
            anyhow::anyhow!(format!(
                "Expire command invalid arguments: cannot parse to integer"
            ))
        })?;

        let metadata = Metadata::try_from(seconds).unwrap();
        let timestamp = self
            .db
            .lock()
            .unwrap()
            .set_expire(&key, metadata)
            .unwrap_or(0);

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
        let result = result
            .into_iter()
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

                let result = result
                    .into_iter()
                    .map(|value| Operation::Bulk(value))
                    .collect();

                Ok(Operation::Array(result))
            }
            any_command => Err(anyhow::anyhow!("Unexpected CONFIG command: {any_command}")),
        }
    }

    fn info(&self, command: Command) -> Result<Operation> {
        let command = command.single_argument()?;
        let (header, info) = match command.as_str() {
            "replication" => {
                let replicator = self.replicator.lock().unwrap();
                let header = "Replication";
                let info = [
                    ("role", self.config.repl_role.to_string()),
                    ("connected_slaves", replicator.slaves_count().to_string()),
                    ("master_replid", replicator.get_master().id.to_string()),
                    ("master_replid2", "0000000000000000000000000000000000000000".to_string()),
                    ("master_repl_offset", "0".to_string()),
                    ("second_repl_offset", "-1".to_string()),
                    ("repl_backlog_active", "0".to_string()),
                    ("repl_backlog_size", "1048576".to_string()),
                    ("repl_backlog_first_byte_offset", "0".to_string()),
                    ("repl_backlog_histlen", "0".to_string()),
                ];

                (header, info)
            }
            any_command => return Err(anyhow::anyhow!("Unexpected INFO command: {any_command}")),
        };

        let formatted = [
            format!("# {header}\n"),
            info.into_iter()
                .map(|(key, value)| format!("{key}:{value}\n"))
                .collect::<String>(),
        ]
        .into_iter()
        .collect::<String>();

        Ok(Operation::Bulk(formatted))
    }

    fn replconf(&self, command: Command) -> Result<Operation> {
        let (command, config) = command.first_2_arguments()?;
        match command.as_str() {
            "listening-port" => {
                let address = format!("127.0.0.1:{config}");
                self.replicator.lock().unwrap().join_slave(address)?;
                Ok(Operation::String(responses::OK.to_string()))
            },
            "capa" => {
                Ok(Operation::String(responses::OK.to_string()))
            },
            any_command => Err(anyhow::anyhow!("Unexpected REPLCONF command: {any_command}")),
        }
    }

    fn psync(&self, command: Command) -> Result<Operation> {
        let (_master_id, offset) = command.first_2_arguments()?;
        let _offset = offset.parse::<i32>()?;
        let master_id = self.config.repl_id.clone();

        Ok(Operation::Sequential(vec![
            Operation::String(format!("FULLRESYNC {master_id} 0")),
            Operation::File(self.config.rdb_empty_file.clone()),
        ]))
    }
}