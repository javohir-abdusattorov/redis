use std::time::Duration;


pub struct Config {
    pub host: String,
    pub port: String,
    pub interval_expiration_enabled: bool,
    pub expiration_min_percent: u8,
    pub expiration_runtime: Duration,
    pub expiration_min_interval: Duration,
    pub expiration_max_interval: Duration,
}

impl Config {
    pub fn build() -> Config {
        Config {
            host: "127.0.0.1".to_string(),
            port: "6378".to_string(),
            interval_expiration_enabled: true,
            expiration_min_percent: 25,
            expiration_runtime: Duration::from_secs(1),
            expiration_min_interval: Duration::from_secs(5),
            expiration_max_interval: Duration::from_secs(60),
        }
    }
}