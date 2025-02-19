use std::collections::HashMap;
use anyhow::Result;
use itertools::Itertools;
use crate::operation::Operation;
use crate::metadata::Metadata;


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
        self.storage.insert(key.clone(), value);
        self.metadata.insert(key.clone(), metatada);
        self.keys.push(key.clone());
    }

    pub fn get(&mut self, key: &String) -> Option<String> {
        if let Some(metadata) = self.metadata.get(key) {
            if metadata.is_expired() {
                self.storage.remove(key);
                return None;
            }
        }

        self.storage.get(key).map(|value| value.clone())
    }
}
