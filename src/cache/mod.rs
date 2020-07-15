pub mod error;
pub mod response;

use chashmap::CHashMap;
use response::{Cache as CacheResponse, CacheStatus};
use std::time::{Duration, Instant};

pub type CacheResult<T> = Result<T, error::CacheError>;

#[derive(Clone)]
pub struct EntryMeta {
    updated_at: Instant,
    frequency: u64,
}

impl EntryMeta {
    pub fn new() -> EntryMeta {
        EntryMeta {
            updated_at: Instant::now(),
            frequency: 0,
        }
    }

    pub fn age(&self) -> Duration {
        self.updated_at.elapsed()
    }
}

#[derive(Clone)]
pub struct Cache {
    max_age: Duration,
    entries: CHashMap<String, Vec<u8>>,
    meta: CHashMap<String, EntryMeta>,
}

impl Cache {
    pub fn new(max_age: Duration) -> Cache {
        Cache {
            max_age,
            entries: CHashMap::new(),
            meta: CHashMap::new(),
        }
    }

    pub fn insert(&self, key: &str, value: Vec<u8>) -> Option<Vec<u8>> {
        self.meta.insert(key.to_string(), EntryMeta::new());
        self.entries.insert(key.to_string(), value)
    }

    pub async fn force_update(&self, key: &str) -> CacheResult<Option<Vec<u8>>> {
        let bytes = reqwest::get(&key.to_string())
            .await?
            .bytes()
            .await?
            .to_vec();

        Ok(self.insert(key, bytes))
    }

    fn get_entry(&self, key: &str) -> Option<Vec<u8>> {
        self.entries.get(key).map(|entry| entry.to_vec())
    }

    pub async fn get(&self, key: &str) -> CacheResult<CacheResponse> {
        let mut status = CacheStatus::Miss;

        if let Some(mut meta) = self.meta.get_mut(key) {
            if meta.age() <= self.max_age {
                if let Some(entry) = self.get_entry(key) {
                    status = CacheStatus::Hit;
                    meta.frequency += 1;

                    return Ok(CacheResponse {
                        bytes: entry.to_vec(),
                        status,
                    });
                }
            } else {
                status = CacheStatus::Expired;
            }
        }

        self.force_update(&key.to_string()).await?;

        Ok(CacheResponse {
            bytes: self.get_entry(key).unwrap(),
            status,
        })
    }
}
