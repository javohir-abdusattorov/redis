use std::time::{Instant, SystemTime, UNIX_EPOCH};
use anyhow::Result;
use itertools::Itertools;
use crate::operation::Operation;


pub struct Metadata {
    expire: u128
}

impl TryFrom<Vec<String>> for Metadata {
    type Error = anyhow::Error;

    fn try_from(operations: Vec<String>) -> Result<Self> {
        let expire = if operations.is_empty() {
            u128::MAX
        }
        else {
            let (key, value) = match operations
                .into_iter()
                .next_tuple()
                {
                    Some((key, value)) => (key, value),
                    None => return Err(anyhow::anyhow!("[Metadata] Invalid arguments")),
                };

            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            let parsed = value.parse::<u128>()?;
            match key.as_str() {
                "EX" => now + (parsed * 1000),
                "PX" => now + parsed,
                _ => return Err(anyhow::anyhow!("[Metadata] Expire time parameter should be EX|PX")),
            }
        };

        Ok(Metadata {
            expire: expire,
        })
    }
}

impl Metadata {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        self.expire <= now
    }
}