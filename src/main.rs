use std::sync::{Arc, Mutex};
use config::Config;
use db::Database;
use expiration::Expiration;
use metadata::Metadata;
use server::Server;

mod config;
mod db;
mod server;
mod resp;
mod operation;
mod metadata;
mod parser;
mod serializer;
mod expiration;

fn main() {
    let config = Arc::new(Config::build());
    let db = Arc::new(Mutex::new(Database::new()));

    populate(Arc::clone(&db));

    Expiration::new(Arc::clone(&config), Arc::clone(&db)).run();
    Server::new(Arc::clone(&config), Arc::clone(&db)).start();

    std::thread::park();
}

fn populate(db: Arc<Mutex<Database>>) {
    use rand::{distr::Alphanumeric, Rng};

    let mut db = db.lock().unwrap();
    let n = 1000;
    (0..n).for_each(|_| {
        let str = rand::rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect::<String>();
        db.set(&str, "1".to_string(), Metadata::try_from(5).unwrap());
    });
}