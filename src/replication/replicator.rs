use super::{member::ReplicationMember, role::ReplicationRole};
use crate::{config::Config, operation::operation::Operation, server::{client::Client, constants}};
use anyhow::Result;
use bytes::BytesMut;
use itertools::Itertools;
use std::{fs::File, io::Write, path::Path, sync::Arc};


pub struct Replicator {
    config: Arc<Config>,
    role: ReplicationRole,
    master: ReplicationMember,
    slaves: Vec<ReplicationMember>,
}

impl Replicator {
    pub fn new(config: Arc<Config>) -> Self {
        let master = ReplicationMember::new(
            ReplicationRole::Master,
            config.repl_id.to_string(),
            format!("{}:{}", config.host, config.port)
        );

        Replicator {
            role: config.repl_role,
            config,
            master,
            slaves: Vec::default(),
        }
    }

    pub fn is_slave(&self) -> bool {
        self.role == ReplicationRole::Slave
    }

    pub fn is_master(&self) -> bool {
        self.role == ReplicationRole::Master
    }

    pub fn get_master(&self) -> &ReplicationMember {
        &self.master
    }

    pub fn handshake_to_master(&mut self) -> Result<()> {
        if self.is_master() {
            return Ok(());
        }

        let address = self.config.repl_master_address.clone();
        let mut client = Client::connect(address.clone())?;

        let response = client.send(vec![
            constants::commands::PING.to_string(),
        ])?.only_string()?;
        println!("[Handshake] PING response = {response:?}");

        let response = client.send(vec![
            constants::commands::REPLCONF.to_string(),
            "listening-port".to_string(),
            self.config.get_by_key(&"port".to_string()).unwrap(),
        ])?.only_string()?;
        println!("[Handshake] REPLCONF response = {response:?}");

        let response = client.send(vec![
            constants::commands::REPLCONF.to_string(),
            "capa".to_string(),
            "psync2".to_string(),
        ])?.only_string()?;
        println!("[Handshake] REPLCONF response = {response:?}");

        let response = client.send(vec![
            constants::commands::PSYNC.to_string(),
            "?".to_string(),
            "-1".to_string(),
        ])?.only_string()?;
        println!("[Handshake] PSYNC response = {response:?}");
        let (master_id, _offset) = response.split(" ").skip(1).next_tuple().unwrap();

        self.master = ReplicationMember::new(
            ReplicationRole::Master, 
            master_id.to_string(),
            address.to_string(),
        );

        let file_buf = client.receive_file()?;
        let rdb_path = Path::new(&self.config.rdb_dir).join(&self.config.rdb_file);
        let mut rdb_file = File::create(rdb_path.clone())?;
        let written = rdb_file.write(file_buf.as_slice())?;
        println!("[Handshake] RDB file received = {rdb_path:?}; written = {written}");

        Ok(())
    }

    pub fn join_slave(&mut self, address: String) -> Result<()> {
        if self.is_slave() {
            return Err(anyhow::anyhow!("Cannot join slave to slave, connect to master"));
        }

        let slave = ReplicationMember::new(
            ReplicationRole::Slave,
            String::new(),
            address,
        );
        self.slaves.push(slave);
        Ok(())
    }

    pub fn slaves_count(&self) -> usize {
        self.slaves.len()
    }
}