use config::{Config, ConfigError, Environment, File};
use humansize::{file_size_opts, FileSize};
use regex::Regex;
use serde::Deserialize;
use std::fmt;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Deserialize, Clone)]
pub struct Fetch {
    #[serde(with = "serde_regex")]
    pub allowed_hosts: Regex,
    pub max_size: u64,
}

impl fmt::Display for Fetch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "- Allowed hosts: {}\n- Maximum download size: {}",
            self.allowed_hosts,
            self.max_size.file_size(file_size_opts::BINARY).unwrap()
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
        writeln!(f, "- Listen address: {}", self.socket_addr())
    }
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Transform {
    pub limits: crate::transform::limit::DimensionLimits,
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "- Limits: {:?}", self.limits)
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub fetch: Fetch,
    pub server: Server,
    pub transform: Transform,
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Fetch settings:\n{}\nServer settings:\n{}Transform settings:\n{}",
            self.fetch, self.server, self.transform
        )
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        println!("Reading configuration file from Settings.toml:");

        s.merge(File::with_name("Settings"))?
            .merge(Environment::with_prefix("PXCMPRS").separator("__"))?;

        let config: Settings = s.try_into()?;

        println!("{}", config);

        Ok(config)
    }
}
