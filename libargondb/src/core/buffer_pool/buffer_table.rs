use std::{collections::HashMap, sync::Mutex};

/// Provides mapping from page tags to page ids. Assumption is that functions
/// provided for this structures are safe without race-conditions.
///
/// TODO: Currently implemented internally as single hashmap guarded by single mutex,
/// which should probably be changed to sharded implementation
#[derive(Debug)]
pub(crate) struct BufferTable<T> {
    inner: Mutex<HashMap<T, usize>>,
}

impl<T> BufferTable<T> {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}
