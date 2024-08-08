use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ConfigError {
    msg: String,
}

impl ConfigError {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        Self { msg: msg.into() }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConfigError: {}", self.msg)
    }
}

impl Error for ConfigError {}
