use crate::operation::operation::Operation;
use super::replicator::Replicator;
use std::{collections::HashMap, io::Write, net::TcpStream, sync::{mpsc::Receiver, Arc, Mutex}};
use anyhow::Result;


pub struct Distributor {
    replicator: Arc<Mutex<Replicator>>,
    channel: Arc<Mutex<Receiver<Operation>>>,
    streams: HashMap<String, TcpStream>,
    initialized: bool,
}

impl Distributor {
    pub fn new(replicator: Arc<Mutex<Replicator>>, channel: Arc<Mutex<Receiver<Operation>>>) -> Self {
        Distributor {
            replicator,
            channel,
            streams: HashMap::new(),
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
            self.replicator
                .lock()
                .unwrap()
                .get_slaves()
                .into_iter()
                .map(|slave| slave.address())
                .map(|address| -> Result<(String, TcpStream)> {
                    match self.streams.get(&address) {
                        None => {
                            let stream = TcpStream::connect(address.clone())?;
                            self.streams.insert(address.clone(), stream.try_clone()?);
                            Ok((address.clone(), stream))
                        },
                        Some(stream) => Ok((address, stream.try_clone()?))
                    }
                })
                .for_each(|stream| {
                    match stream {
                        Ok((address, mut stream)) => {
                            println!("SENDING THIS TO NIGGA - {address}:");
                            println!("{event:?}");
                            stream.write(&event.to_bytes()).unwrap();
                        }
                        Err(err) => {
                            println!("Cannot send to this nigga:");
                            println!("{err}");
                        }
                    }
                });
        }
    }
}