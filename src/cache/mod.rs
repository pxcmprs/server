pub mod error;
pub mod response;

use chashmap::CHashMap;
use response::{Cache as CacheResponse, CacheStatus};
use std::time::{Duration, Instant};
use url::Url;

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
    allowed_hosts: regex::Regex,
}

impl Cache {
    pub fn new(max_age: Duration, allowed_hosts: regex::Regex) -> Cache {
        Cache {
            max_age,
            entries: CHashMap::new(),
            meta: CHashMap::new(),
            allowed_hosts,
        }
    }

    fn normalize_key(url: &Url) -> String {
        format!(
            "{}{}?{}",
            url.host_str().unwrap_or(""),
            url.path(),
            url.query().unwrap_or("")
        )
    }

    pub fn insert(&self, url: &Url, value: Vec<u8>) -> Option<Vec<u8>> {
        let normalized = Cache::normalize_key(url);
        self.meta.insert(normalized.to_string(), EntryMeta::new());
        self.entries.insert(normalized, value)
    }

    pub async fn force_update(&self, url: &Url) -> CacheResult<Option<Vec<u8>>> {
        if let Some(host) = url.host_str() {
            if self.allowed_hosts.is_match(&host) {
                let bytes = reqwest::get(&url.to_string())
                    .await?
                    .bytes()
                    .await?
                    .to_vec();

                Ok(self.insert(url, bytes))
            } else {
                Err(error::CacheError::IllegalHost(host.to_string()))
            }
        } else {
            Err(error::CacheError::IllegalHost("".to_string()))
        }
    }

    fn get_entry(&self, url: &Url) -> Option<Vec<u8>> {
        self.entries
            .get(&Cache::normalize_key(url))
            .map(|entry| entry.to_vec())
    }

    pub async fn get(&self, url: &Url) -> CacheResult<CacheResponse> {
        let mut status = CacheStatus::Miss;

        if let Some(mut meta) = self.meta.get_mut(&Cache::normalize_key(url)) {
            if meta.age() <= self.max_age {
                if let Some(entry) = self.get_entry(url) {
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

        self.force_update(&url).await?;

        Ok(CacheResponse {
            bytes: self.get_entry(url).unwrap(),
            status,
        })
    }
}
