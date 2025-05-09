use crate::{config::Config, operation::operation::Operation, server::{client::Client, constants::commands}};
use super::replicator::Replicator;
use std::{collections::HashMap, io::{Read, Write}, net::TcpStream, sync::{mpsc::Receiver, Arc, Mutex}, time::Duration};
use anyhow::Result;


pub struct HealthChecker {
    replicator: Arc<Mutex<Replicator>>,
    config: Arc<Config>,
    initialized: bool,
}

impl HealthChecker {
    pub fn new(config: Arc<Config>, replicator: Arc<Mutex<Replicator>>) -> Self {
        HealthChecker {
            replicator,
            config,
            initialized: false,
        }
    }

    pub fn run(mut self) {
        if self.initialized || self.replicator.lock().unwrap().is_slave() {
            return;
        }

        self.initialized = true;
        std::thread::Builder::new()
            .name("health-check".into())
            .spawn(move || self.check())
            .unwrap();
    }

    fn check(mut self) {
        let mut buffer = [0; 512];
        let acknowledgement_request: Vec<String> = vec![
            commands::REPLCONF.into(),
            "getack".into(),
            "*".into(),
        ];

        let mut acknowledgement = |address: &String, stream: &mut TcpStream| {
            match Client::send_ref(stream, &mut buffer, acknowledgement_request.clone()) {
                Err(err) => {
                    println!("[Health] Slave {address:?} malfunctioning: {err}");
                },
                Ok(response) => {
                    let (command, arguments) = response.only_array().unwrap();
                    let offset = arguments[1].clone().only_bulk().unwrap().parse::<u32>().unwrap();
                    println!("[Health] Slave {address:?} offset: {offset}");
                }
            };
        };

        self.replicator
            .lock()
            .unwrap()
            .get_slaves()
            .into_iter()
            .map(|(address, slave)| {
                match slave.connect() {
                    Err(err) => eprintln!("[Health] Slave {:?} cannot be connected", slave.address),
                    Ok(stream) => acknowledgement(address, stream.as_mut().unwrap())
                }
            })
            .for_each(|_| {});

        std::thread::sleep(self.config.repl_health_check_interval);
        self.check();
    }
}