use serde::Deserialize;
use std::{fs, io, ops::Deref};

use super::error::ConfigError;

pub fn get_default() -> Config {
    Config::default()
}

pub fn try_read_config() -> Result<Config, ConfigError> {
    let path = "/etc/argon-db/config.toml";
    let config_str = fs::read_to_string(path).map_err(map_io_error)?;

    toml::from_str(&config_str).map_err(|err| {
        ConfigError::new(format!(
            "Error parsing content of config file. Error: {}",
            err
        ))
    })
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub logs: Logs,

    #[serde(default)]
    pub execution_pool_size: ExecPoolSize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logs {
    log_file: String,
}

impl Default for Logs {
    fn default() -> Self {
        Self {
            log_file: String::from("/var/log/argon-db.log"),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ExecPoolSize(usize);

impl Default for ExecPoolSize {
    fn default() -> Self {
        Self(num_cpus::get())
    }
}

impl Into<usize> for ExecPoolSize {
    fn into(self) -> usize {
        self.0
    }
}

fn map_io_error(err: io::Error) -> ConfigError {
    match err.kind() {
        io::ErrorKind::NotFound => {
            ConfigError::new("Could not find config file in expected location")
        }
        _ => ConfigError::new(format!("Error reading config file. Error: {}", err)),
    }
}
