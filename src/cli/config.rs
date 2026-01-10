use std::{env, error::Error};

use serde::{Deserialize, Serialize};

use akv::utils::resolve_config_path;

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
    "127.0.0.1".into()
}

fn default_port() -> u16 {
    6000
}

fn default_string() -> String {
    "".into()
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let args: Vec<String> = env::args().collect();
        let mut config = Self::default();

        let mut config_file = None;
        let mut i = 1;

        while i < args.len() {
            match args[i].as_str() {
                "-c" | "--config" => {
                    if i + 1 < args.len() {
                        config_file = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return Err("Missing config file path after -c/--config".into());
                    }
                }
                "-h" | "--host" => {
                    if i + 1 < args.len() {
                        config.host = args[i + 1].clone();
                        i += 2;
                    } else {
                        return Err("Missing host after -h/--host".into());
                    }
                }
                "-p" | "--port" => {
                    if i + 1 < args.len() {
                        config.port = args[i + 1].parse()?;
                        i += 2;
                    } else {
                        return Err("Missing port after -p/--port".into());
                    }
                }
                "-s" | "--secret" => {
                    if i + 1 < args.len() {
                        config.secret = args[i + 1].clone();
                        i += 2;
                    } else {
                        return Err("Missing secret after -s/--secret".into());
                    }
                }
                "--help" => {
                    Self::print_help();
                    std::process::exit(0);
                }
                _ => {
                    i += 1;
                }
            }
        }

        if let Some(ref file) = config_file {
            match resolve_config_path(file) {
                Ok(abs_path) => {
                    if abs_path.exists() && abs_path.is_file() {
                        let content = std::fs::read_to_string(abs_path)?;
                        let file_config: Config = toml::from_str(&content)?;
                        if config.host == default_host() {
                            config.host = file_config.host;
                        }
                        if config.port == default_port() {
                            config.port = file_config.port;
                        }
                        if config.secret == default_string() {
                            config.secret = file_config.secret;
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to resolve config file path: {}", e).into());
                }
            }
        }

        config.parse_env_var();
        Ok(config)
    }

    pub fn parse_env_var(&mut self) {
        if let Ok(host) = std::env::var("AKV_HOST") {
            self.host = host;
        }
        if let Ok(port) = std::env::var("AKV_PORT") {
            self.port = port.parse().unwrap();
        }
        if let Ok(secret) = std::env::var("AKV_SECRET") {
            self.secret = secret;
        }
    }

    pub fn get_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn print_help() {
        println!("Usage: akvc [OPTIONS]");
        println!();
        println!("Options:");
        println!("  -c, --config <file>   Path to config file (default: ./config.toml)");
        println!("  -h, --host <host>     Server host (default: 127.0.0.1)");
        println!("  -p, --port <port>     Server port (default: 6000)");
        println!("  -s, --secret <secret> Authentication secret");
        println!("      --help             Show this help message");
        println!();
        println!("Environment variables:");
        println!("  AKV_HOST              Server host");
        println!("  AKV_PORT              Server port");
        println!("  AKV_SECRET            Authentication secret");
        println!();
        println!("Examples:");
        println!("  akvc -c config.toml");
        println!("  akvc -h 127.0.0.1 -p 6000 -s my_secret");
        println!("  akvc --host localhost --port 6000 --secret my_secret");
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
