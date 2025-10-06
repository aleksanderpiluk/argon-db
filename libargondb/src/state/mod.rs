mod db;
mod table;

use std::sync::{Arc, Mutex};

pub struct InstanceState {}

struct StateRCUNode<T> {
    write_lock: Mutex<()>,
    value: Mutex<Arc<T>>,
}

impl<T> StateRCUNode<T> {}
