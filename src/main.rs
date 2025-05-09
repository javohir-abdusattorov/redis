#![allow(warnings)]

use std::sync::{Arc, Mutex};
use config::Config;
use expiration::Expiration;
use operation::metadata::Metadata;
use replication::{distributor::Distributor, health_checker::HealthChecker, replicator::Replicator};
use server::server::Server;
use storage::{db::Database, parser::Parser};

mod operation;
mod server;
mod storage;
mod config;
mod replication;
mod expiration;

fn main() {
    let config = Arc::new(Config::build());
    let db: Arc<Mutex<Database>> = Arc::new(Mutex::new(Database::new()));
    populate(Arc::clone(&db));

    let (mut replicator, channel) = Replicator::new(Arc::clone(&config));
    replicator.handshake_to_master().unwrap();
    let channel = Arc::new(Mutex::new(channel));
    let replicator = Arc::new(Mutex::new(replicator));

    Distributor::new(Arc::clone(&replicator), Arc::clone(&channel)).run();
    HealthChecker::new(Arc::clone(&config), Arc::clone(&replicator)).run();
    Parser::new(Arc::clone(&config), Arc::clone(&db)).parse().unwrap();
    Expiration::new(Arc::clone(&config), Arc::clone(&db)).run();
    Server::new(Arc::clone(&config), Arc::clone(&db), Arc::clone(&replicator)).start();

    std::thread::park();
}

fn populate(db: Arc<Mutex<Database>>) {
    use rand::{distr::Alphanumeric, Rng};

    let mut db = db.lock().unwrap();
    let n = 1;
    (0..n).for_each(|_| {
        let str = rand::rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect::<String>();
        db.set(&str, "1".to_string(), Metadata::try_from(1500).unwrap());
    });
}