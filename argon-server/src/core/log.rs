use std::fmt::Display;

pub fn log_bootstrap_err<T: Display>(error: T) {
    println!("Error bootstrapping ArgonDB: {}", error);
}
