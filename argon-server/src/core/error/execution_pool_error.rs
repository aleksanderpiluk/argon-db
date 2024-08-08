use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ExecPoolError {
    msg: String,
}

impl ExecPoolError {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        Self { msg: msg.into() }
    }
}

impl fmt::Display for ExecPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExecutionPoolError: {}", self.msg)
    }
}

impl Error for ExecPoolError {}
