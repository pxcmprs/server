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

#[derive(Clone, Debug)]
pub struct CacheOptions {
    pub max_age: Duration,
    pub max_entry_size: u64,
    pub table_size: usize,
    pub allowed_hosts: regex::Regex,
}

impl From<crate::settings::Fetch> for CacheOptions {
    fn from(settings: crate::settings::Fetch) -> Self {
        Self {
            max_age: Duration::from_secs(settings.cache.max_age),
            max_entry_size: settings.max_size,
            table_size: settings.cache.max_entries,
            allowed_hosts: settings.allowed_hosts,
        }
    }
}

#[derive(Clone)]
pub struct Cache {
    entries: CHashMap<String, Vec<u8>>,
    meta: CHashMap<String, EntryMeta>,
    options: CacheOptions,
}

impl Cache {
    pub fn new<O: Into<CacheOptions>>(options: O) -> Cache {
        let options = options.into();

        Cache {
            entries: CHashMap::with_capacity(options.table_size),
            meta: CHashMap::with_capacity(options.table_size),
            options,
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

    fn assert_within_size_limit(&self, size: u64) -> CacheResult<()> {
        if size > self.options.max_entry_size {
            // The response is larger than the maximum allowed size. ERROR!!!
            Err(error::CacheError::MaxSizeExceeded(
                self.options.max_entry_size,
                size,
            ))
        } else {
            Ok(())
        }
    }

    pub async fn force_update(&self, url: &Url) -> CacheResult<Option<Vec<u8>>> {
        if let Some(host) = url.host_str() {
            if self.options.allowed_hosts.is_match(&host) {
                let res = reqwest::get(&url.to_string()).await?;

                let body_size = match res.content_length() {
                    Some(x) => x,
                    None => {
                        // If Reqwest can't determine the size of the input, nobody can! We must play it safe and ABORT!
                        return Err(error::CacheError::InvalidInput);
                    }
                };

                self.assert_within_size_limit(body_size)?;

                let bytes = res.bytes().await?.to_vec();

                self.assert_within_size_limit(bytes.len() as u64)?;

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
            if meta.age() <= self.options.max_age {
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
