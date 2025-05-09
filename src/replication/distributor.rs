use crate::{operation::operation::Operation, server::{client::Client, constants::commands}};
use super::replicator::Replicator;
use std::{collections::HashMap, io::Write, net::TcpStream, sync::{mpsc::Receiver, Arc, Mutex}, time::Duration};
use anyhow::Result;


pub struct Distributor {
    replicator: Arc<Mutex<Replicator>>,
    channel: Arc<Mutex<Receiver<Operation>>>,
    streams: Arc<Mutex<HashMap<String, TcpStream>>>,
    initialized: bool,
}

impl Distributor {
    pub fn new(replicator: Arc<Mutex<Replicator>>, channel: Arc<Mutex<Receiver<Operation>>>) -> Self {
        Distributor {
            replicator,
            channel,
            streams: Arc::new(Mutex::new(HashMap::new())),
            initialized: false,
        }
    }

    pub fn run(mut self) {
        if self.initialized || self.replicator.lock().unwrap().is_slave() {
            return;
        }

        self.initialized = true;
        std::thread::Builder::new()
            .name("distributor".into())
            .spawn(move || self.distribute())
            .unwrap();
    }

    fn distribute(mut self) {
        for event in self.channel.lock().unwrap().iter() {
            let bytes = event.to_bytes();
            self.replicator
                .lock()
                .unwrap()
                .get_slaves()
                .into_iter()
                .filter_map(|(_, slave)| slave.connect().ok())
                .for_each(|slave| {
                    slave.as_mut().unwrap().write(&bytes).unwrap();
                    slave.as_mut().unwrap().flush().unwrap();
                });
        }
    }
}