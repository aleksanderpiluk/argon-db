use std::{fmt::Debug, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use async_lock::Mutex;

pub struct RCU<T> {
    state_mut_lock: Mutex<()>,
    state: ArcSwap<T>,
}

impl<T> RCU<T> {
    pub fn new(state: Arc<T>) -> Self {
        Self {
            state_mut_lock: Mutex::new(()),
            state: ArcSwap::new(state),
        }
    }

    pub fn load(&self) -> impl Deref<Target = Arc<T>> {
        self.state.load()
    }

    pub async fn mutate<F>(&self, mutate_fn: F)
    where
        F: FnOnce(&T) -> Option<T>,
    {
        let _guard = self.state_mut_lock.lock().await;

        let current = self.state.load();
        if let Some(next) = mutate_fn(&current) {
            self.state.store(Arc::new(next));
        }
    }

    pub fn mutate_blocking<F>(&self, mutate_fn: F) -> bool
    where
        F: FnOnce(&T) -> Option<T>,
    {
        let _guard = self.state_mut_lock.lock_blocking();

        let current = self.state.load();
        if let Some(next) = mutate_fn(&current) {
            self.state.store(Arc::new(next));
            true
        } else {
            false
        }
    }
}

impl<T: Debug> Debug for RCU<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RCU")
            .field("state", &self.state.load_full())
            .finish()
    }
}
