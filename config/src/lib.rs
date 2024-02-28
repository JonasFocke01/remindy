use std::{fs, net::Ipv4Addr, str::FromStr};

use serde::Deserialize;

use reminder::root_path;

const CONFIG_FILE_NAME: &str = "remindy.toml";

#[derive(Deserialize, Default, Clone)]
pub struct Config {
    network: Network,
}

impl Config {
    pub fn new() -> Self {
        if let Ok(root_path) = root_path() {
            let raw_config = fs::read_to_string(
                format!("{:?}/{CONFIG_FILE_NAME}", root_path)
                    .replace("\"", "")
                    .as_str(),
            )
            .expect(format!("Config not found in {:?}/{CONFIG_FILE_NAME}", root_path).as_str());
            let result = toml::from_str(&raw_config).expect(
                format!(
                    "Config file in {:?}/{CONFIG_FILE_NAME} is invalid",
                    root_path
                )
                .as_str(),
            );
            return result;
        } else {
            panic!("home_dir not found")
        }
    }
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
    pub fn local_ip(&self) -> &String {
        &self.local_ip
    }
    pub fn local_ip_as_ipv4(&self) -> Ipv4Addr {
        Ipv4Addr::from_str(&self.local_ip).expect("Could not format local_ip as Ipv4Addr")
    }
    pub fn remote_ip(&self) -> &String {
        if let Some(remote_ip) = &self.remote_ip {
            &remote_ip
        } else {
            &self.local_ip
        }
    }
    pub fn port(&self) -> &String {
        &self.port
    }
    pub fn port_as_u16(&self) -> u16 {
        self.port.parse().unwrap()
    }
}
