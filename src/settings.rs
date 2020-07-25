use config::{Config, ConfigError, File};
use humansize::{file_size_opts, FileSize};
use regex::Regex;
use serde::Deserialize;
use std::fmt;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Deserialize)]
pub struct Cache {
    pub max_age: u64,
    pub max_entries: usize,
}

#[derive(Debug, Deserialize)]
pub struct Fetch {
    #[serde(with = "serde_regex")]
    pub allowed_hosts: Regex,
    pub max_size: u64,
    pub cache: Cache,
}

impl fmt::Display for Fetch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "- Allowed hosts: {}\n- Maximum download size: {}\n- Maximum cache entry age: {}\n- Maximum number of cached entries: {}",
            self.allowed_hosts,
            self.max_size.file_size(file_size_opts::BINARY).unwrap(),
            self.cache.max_age,
            self.cache.max_entries
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub address: IpAddr,
    pub port: u16,
}

impl Server {
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "- Listed address: {}", self.socket_addr())
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub fetch: Fetch,
    pub server: Server,
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Fetch settings:\n{}\nServer settings:\n{}",
            self.fetch, self.server
        )
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        println!("Reading configuration file from Settings.toml:");

        s.merge(File::with_name("Settings").required(true))?;

        let config: Settings = s.try_into()?;

        println!("{}", config);

        Ok(config)
    }
}
