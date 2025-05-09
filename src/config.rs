use crate::replication::role::ReplicationRole;
use rand::Rng;
use std::{collections::HashMap, time::Duration};


pub struct Config {
    pub host: String,
    pub port: String,
    pub interval_expiration_enabled: bool,
    pub expiration_min_percent: u8,
    pub expiration_runtime: Duration,
    pub expiration_min_interval: Duration,
    pub expiration_max_interval: Duration,
    pub rdb_dir: String,
    pub rdb_file: String,
    pub rdb_empty_file: String,
    pub repl_role: ReplicationRole,
    pub repl_master_address: String,
    pub repl_id: String,
    pub repl_health_check_interval: Duration,
    pub key_map: HashMap<String, String>,
}

impl Config {
    pub fn build() -> Config {
        let mut config = Config {
            host: "127.0.0.1".to_string(),
            port: "6378".to_string(),
            interval_expiration_enabled: false,
            expiration_min_percent: 25,
            expiration_runtime: Duration::from_secs(1),
            expiration_min_interval: Duration::from_secs(5),
            expiration_max_interval: Duration::from_secs(60),
            rdb_dir: "/home/javohir/Downloads/Temp/rdb".to_string(),
            rdb_file: "dump.rdb".to_string(),
            rdb_empty_file: "/home/javohir/Documents/Programming/Learning/Build Your Own/redis/static/empty.rdb".to_string(),
            repl_role: ReplicationRole::Master,
            repl_id: rand::rng()
                .sample_iter(rand::distr::Alphanumeric)
                .take(40)
                .map(char::from)
                .collect(),
            repl_master_address: String::new(),
            repl_health_check_interval: Duration::from_secs(20),
            key_map: HashMap::default(),
        };

        config.parse_from_args();
        config.build_key_map();

        config
    }

    pub fn get_by_key(&self, key: &String) -> Option<String> {
        self.key_map.get(key).map(|value| value.clone())
    }

    fn parse_from_args(&mut self) {
        std::env::args()
            .skip(1)
            .step_by(2)
            .zip(std::env::args().skip(2).step_by(2))
            .map(|(key, value)| (key.replace("--", ""), value))
            .for_each(|(key, value)| {
                match key.as_str() {
                    "port" => self.port = value,
                    "host" => self.host = value,
                    "replicaof" => {
                        self.repl_role = ReplicationRole::Slave;
                        self.repl_master_address = value.replace(" ", ":");
                    },
                    _ => {}
                }
            });
    }

    fn build_key_map(&mut self) {
        self.key_map = HashMap::from([
            ("host".to_string(), self.host.clone()),
            ("port".to_string(), self.port.clone()),
            ("expiration_enabled".to_string(), self.interval_expiration_enabled.to_string().clone()),
            ("dir".to_string(), self.rdb_dir.clone()),
            ("dbfilename".to_string(), self.rdb_file.clone()),
        ]);
    }
}
