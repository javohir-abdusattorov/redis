use std::sync::{Arc, Mutex};
use config::Config;
use db::Database;
use expiration::Expiration;
use metadata::Metadata;
use tokio::{net::TcpListener, runtime::Runtime};
use resp::RespHandler;

mod config;
mod db;
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

    let expire_config = Arc::clone(&config);
    let expire_db = Arc::clone(&db);
    std::thread::spawn(|| Expiration::new(expire_config, expire_db).run());

    std::thread::spawn(|| {
        let rt = Runtime::new().unwrap();

        rt.spawn(async move {
            let db = Arc::clone(&db);
            let host = [config.host.clone(), config.port.clone()].join(":");
            let listener = TcpListener::bind(host.clone()).await.unwrap();
            println!("Redis server started at host: {host}");

            loop {
                let stream = listener.accept().await;

                match stream {
                    Ok((stream, addr)) => {
                        let db = Arc::clone(&db);
                        println!("connection: {addr:?}");
        
                        tokio::spawn(async move {
                            RespHandler::new(
                                stream,
                                db,
                            )
                            .process().await
                        });
                    }
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
            }
        });
    });
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