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
            key_map: HashMap::default(), 
        };

        config.key_map = HashMap::from([
            ("host".to_string(), config.host.clone()),
            ("port".to_string(), config.port.clone()),
            ("expiration_enabled".to_string(), config.interval_expiration_enabled.to_string().clone()),
            ("dir".to_string(), config.rdb_dir.clone()),
            ("dbfilename".to_string(), config.rdb_file.clone()),
        ]);

        config
    }

    pub fn get_by_key(&self, key: &String) -> Option<String> {
        self.key_map
            .get(key)
            .map(|value| value.clone())
    }
}