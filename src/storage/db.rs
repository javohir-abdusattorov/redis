use std::collections::HashMap;
use itertools::Itertools;
use rand::Rng;
use fast_glob::glob_match;
use crate::operation::metadata::Metadata;


pub struct Database {
    storage: HashMap<String, String>,
    metadata: HashMap<String, Metadata>,
    keys: Vec<String>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            storage: HashMap::default(),
            metadata: HashMap::default(),
            keys: Vec::default(),
        }
    }

    pub fn set(&mut self, key: &String, value: String, metatada: Metadata) {
        self.keys.push(key.clone());
        self.storage.insert(key.clone(), value);
        self.metadata.insert(key.clone(), metatada);
    }

    pub fn get(&mut self, key: &String) -> Option<String> {
        if self.is_expired(key) {
            self.del(key);
            return None;
        }

        self.storage.get(key).map(|value| value.clone())
    }

    pub fn ttl(&mut self, key: &String) -> i128 {
        match self.get(key) {
            None => -2,
            Some(_) => {
                match self.metadata.get(key).unwrap().expire_duration() {
                    None => -1,
                    Some(duration) => duration.as_secs() as i128
                }
            }
        }
    }

    pub fn set_expire(&mut self, key: &String, metadata: Metadata) -> Option<u128> {
        self
            .get(key)
            .map(|_| {
                let timestamp = metadata.expire_timestamp();
                self.metadata.insert(key.clone(), metadata);

                timestamp
            })
    }

    pub fn try_expire(&mut self, key: &String) -> Option<()> {
        if self.is_expired(key) {
            self.del(key);
            Some(())
        }
        else {
            None
        }
    }

    pub fn del(&mut self, key: &String) {
        self.storage.remove(key);
        self.metadata.remove(key);
        self.keys.remove(self.find_position(key).unwrap());
    }

    pub fn search(&mut self, pattern: &String) -> Vec<String> {
        let pattern = pattern.as_str();

        // If requesting all keys
        if pattern == "*" {
            self.keys.clone()
        }

        // If there is not glob pattern, just get by key
        else if !pattern.contains(['*', '?', '[', ']', '-', '^']) {
            self.get(&pattern.to_string())
                .map(|_| vec![pattern.to_string()])
                .unwrap_or(vec![])
        }

        // Glob pattern search
        else {
            self.keys
                .iter()
                .filter(|key| glob_match(pattern, &key))
                .map(|str| str.clone())
                .collect()
        }
    }

    pub fn find_position(&self, key: &String) -> Option<usize> {
        self.keys
            .iter()
            .find_position(|a| a == &key)
            .map(|(i, _)| i)
    }

    pub fn is_expired(&self, key: &String) -> bool {
        self.metadata
            .get(key)
            .map(|metadata| metadata.is_expired())
            .unwrap_or(false)
    }

    pub fn get_random(&self) -> Option<String> {
        if self.keys.is_empty() {
            return None;
        }

        let (min, max) = (0, self.keys.len());
        let random = rand::rng().random_range(min..max);

        Some(self.keys[random].clone())
    }

    pub fn size(&self) -> usize {
        self.storage.len()
    }
}
