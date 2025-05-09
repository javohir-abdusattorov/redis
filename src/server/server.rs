use super::{handler::Handler, router::Router};
use crate::{config::Config, replication::replicator::Replicator};
use crate::storage::db::Database;
use std::{net::SocketAddr, sync::{Arc, Mutex}};
use tokio::{net::{TcpListener, TcpStream}, runtime::Runtime};

pub struct Server {
    config: Arc<Config>,
    db: Arc<Mutex<Database>>,
    replicator: Arc<Mutex<Replicator>>,
}

impl Server {
    pub fn new(config: Arc<Config>, db: Arc<Mutex<Database>>, replicator: Arc<Mutex<Replicator>>) -> Self {
        Server { config, db, replicator }
    }

    pub fn start(self) {
        let server = async move {
            let host = [self.config.host.clone(), self.config.port.clone()].join(":");
            let listener = TcpListener::bind(host.clone()).await.unwrap();
            println!("[Server] Started at: {host}");

            self.streamer(listener).await;
        };

        std::thread::Builder::new()
            .name("server".into())
            .spawn(move || {
                Runtime::new().unwrap().block_on(server);
            })
            .unwrap();
    }

    async fn streamer(self, listener: TcpListener) {
        loop {
            match listener.accept().await {
                Err(err) => {
                    println!("[Server] cannot establish connection: {err}");
                }
                Ok((stream, addr)) => self.handler(stream, addr),
            }
        }
    }

    fn handler(&self, stream: TcpStream, addr: SocketAddr) {
        let config = Arc::clone(&self.config);
        let db = Arc::clone(&self.db);
        let router = Router::new(config, db, Arc::clone(&self.replicator));
        let mut handler = Handler::new(stream, router, Arc::clone(&self.replicator));
        println!("[Server] connection established: {addr:?}");

        tokio::spawn(async move { handler.process().await });
    }
}