use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::utils::resolve_config_path;

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_string")]
    pub secret: String,
}

fn default_host() -> String {
    "0.0.0.0".into()
}
fn default_port() -> u16 {
    60002
}

fn default_string() -> String {
    "".into()
}

impl Config {
    pub fn new(config_path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let config_file = config_path.unwrap_or("./config.toml");

        match resolve_config_path(config_file) {
            Ok(abs_path) => {
                if !abs_path.exists() || !abs_path.is_file() {
                    Ok(Self::default())
                } else {
                    let content = std::fs::read_to_string(abs_path)?;
                    let mut config = toml::from_str::<Config>(&content)?;
                    config.parse_env_var();
                    Ok(config)
                }
            }
            Err(e) => {
                panic!("Failed to resolve config file path: {}", e);
            }
        }
    }

    pub fn parse_env_var(&mut self) {
        if let Ok(host) = std::env::var("AKV_HOST") {
            self.host = host;
        }
        if let Ok(port) = std::env::var("AKV_PORT") {
            self.port = port.parse().unwrap();
        }
    }

    pub fn get_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            secret: default_string(),
        }
    }
}
