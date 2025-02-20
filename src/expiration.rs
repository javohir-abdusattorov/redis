use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::config::Config;
use crate::db::Database;


pub struct Expiration {
    config: Arc<Config>,
    db: Arc<Mutex<Database>>,
    initialized: bool,
}

impl Expiration {
    pub fn new(config: Arc<Config>, db: Arc<Mutex<Database>>) -> Self {
        Expiration {
            config,
            db,
            initialized: false,
        }
    }

    pub fn run(&mut self) {
        if !self.initialized {
            self.expire();
            self.initialized = true;
        }
    }

    fn expire(&self) {
        if !self.config.interval_expiration_enabled {
            return;
        }

        println!("[Expiration] Started");

        let now = Instant::now();
        let total = self.db.lock().unwrap().size() as u32;
        let mut processed: u32 = 0;
        let mut expired: u32 = 0;

        while processed < total && now.elapsed() < self.config.expiration_runtime {
            let mut db = self.db.lock().unwrap();
            match db.get_random() {
                None => break,
                Some(key) => {
                    expired += db.try_expire(&key).map(|_| 1).unwrap_or(0);
                    processed += 1;
                }
            }
        }

        let threshold = ((processed as f64 / 100.0) * self.config.expiration_min_percent as f64) as u32;
        let is_min_expired = expired > threshold;
        let interval = if is_min_expired {
            self.config.expiration_min_interval
        } else {
            self.config.expiration_max_interval
        };

        println!("[Expiration] Sleeping: {interval:?}; elapsed: {:?}; size = {total}; processed = {processed}; expired = {expired}; threshold = {threshold}", now.elapsed());
        std::thread::sleep(interval);
        self.expire();
    }
}