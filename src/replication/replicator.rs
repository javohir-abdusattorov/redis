use super::{member::ReplicationMember, role::ReplicationRole};
use crate::{config::Config, operation::operation::Operation, server::{client::Client, constants}};
use anyhow::Result;
use itertools::Itertools;
use std::{collections::HashMap, fs::File, io::Write, net::TcpStream, path::Path, sync::{mpsc::{Receiver, Sender}, Arc}};


pub struct Replicator {
    config: Arc<Config>,
    role: ReplicationRole,
    master: ReplicationMember,
    slaves: HashMap<String, ReplicationMember>,
    channel: Sender<Operation>,
    offset: u32,
}

impl Replicator {
    pub fn new(config: Arc<Config>) -> (Self, Receiver<Operation>) {
        let master = ReplicationMember::new(
            ReplicationRole::Master,
            config.repl_id.to_string(),
            format!("{}:{}", config.host, config.port)
        );

        let (tx, rv) = std::sync::mpsc::channel::<Operation>();
        let replicator = Replicator {
            role: config.repl_role,
            config,
            master,
            slaves: HashMap::default(),
            channel: tx,
            offset: 0,
        };

        (replicator, rv)
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

    pub fn get_slaves(&mut self) -> &mut HashMap<String, ReplicationMember> {
        &mut self.slaves
    }

    pub fn get_offset(&self) -> u32 {
        self.offset
    }

    pub fn slaves_count(&self) -> usize {
        self.slaves.len()
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
            return Err(anyhow::anyhow!("Cannot join slave to slave, connect to master at: {}", self.master.address));
        }

        self.slaves.insert(
            address.clone(),
            ReplicationMember::new(
                ReplicationRole::Slave,
                String::new(),
                address,
            ),
        );
        Ok(())
    }

    pub fn distribute(&self, message: Operation) -> Result<()> {
        if self.is_master() && self.slaves_count() >= 1 {
            self.channel.send(message)?;
        }

        Ok(())
    }

    pub fn offset(&mut self, bytes: usize) {
        self.offset += bytes as u32;
    }
}