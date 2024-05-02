use std::{fs, net::Ipv4Addr, str::FromStr};

use serde::Deserialize;

use reminder::root_path;

const CONFIG_FILE_NAME: &str = "remindy.toml";

#[derive(Deserialize, Default, Clone)]
pub struct Config {
    network: Network,
}

impl Config {
    #[allow(clippy::panic, clippy::expect_used)]
    #[must_use]
    /// # Panics
    pub fn new() -> Self {
        if let Ok(root_path) = root_path() {
            let raw_config = fs::read_to_string(
                format!("{root_path:?}/{CONFIG_FILE_NAME}")
                    .replace('\"', "")
                    .as_str(),
            )
            .unwrap_or_else(|e| {
                panic!("Config not found in {root_path:?}/{CONFIG_FILE_NAME}\nError {e:?}")
            });
            let result = toml::from_str(&raw_config).unwrap_or_else(|e| {
                panic!("Config file in {root_path:?}/{CONFIG_FILE_NAME} is invalid\nError {e:?}",)
            });
            return result;
        }
        panic!("home_dir not found")
    }
    #[must_use]
    pub fn network(&self) -> &Network {
        &self.network
    }
}

#[derive(Deserialize, Default, Clone)]
pub struct Network {
    remote_ip: Option<String>,
    local_ip: String,
    port: String,
}
impl Network {
    #[must_use]
    pub fn local_ip(&self) -> &String {
        &self.local_ip
    }
    #[must_use]
    #[allow(clippy::expect_used)]
    /// # Panics
    pub fn local_ip_as_ipv4(&self) -> Ipv4Addr {
        Ipv4Addr::from_str(&self.local_ip).expect("Could not format local_ip as Ipv4Addr")
    }
    #[must_use]
    pub fn remote_ip(&self) -> &String {
        if let Some(remote_ip) = &self.remote_ip {
            remote_ip
        } else {
            &self.local_ip
        }
    }
    #[must_use]
    pub fn port(&self) -> &String {
        &self.port
    }
    #[allow(clippy::unwrap_used)]
    #[must_use]
    /// # Panics
    pub fn port_as_u16(&self) -> u16 {
        self.port.parse().unwrap()
    }
}
