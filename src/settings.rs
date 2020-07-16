use config::{Config, ConfigError, File};
use regex::Regex;
use serde::Deserialize;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn default_allowed_hosts() -> Regex {
    Regex::new(".*").unwrap()
}

#[derive(Debug, Deserialize)]
pub struct Fetch {
    #[serde(default = "default_allowed_hosts")]
    #[serde(with = "serde_regex")]
    pub allowed_hosts: Regex,
}

impl Default for Fetch {
    fn default() -> Self {
        Self {
            allowed_hosts: default_allowed_hosts(),
        }
    }
}

fn default_addr() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}

fn default_port() -> u16 {
    8080
}

#[derive(Debug, Deserialize)]
pub struct Server {
    #[serde(default = "default_addr")]
    pub address: IpAddr,

    #[serde(default = "default_port")]
    pub port: u16,
}

impl Server {
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            address: default_addr(),
            port: default_port(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default = "Fetch::default")]
    pub fetch: Fetch,

    #[serde(default = "Server::default")]
    pub server: Server,
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "- Allowed hosts: {}\n- Listen address: {}",
            self.fetch.allowed_hosts,
            self.server.socket_addr()
        )
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        println!("Reading configuration file from Settings.toml:");

        s.merge(File::with_name("Settings").required(false))?;

        let config: Settings = s.try_into()?;

        println!("{}", config);

        Ok(config)
    }
}
