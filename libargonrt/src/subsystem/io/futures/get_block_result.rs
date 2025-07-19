use std::{
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

use crate::foundation::block::BlockTag;

pub struct GetBlockResult {
    shared_state: Arc<Mutex<SharedState>>,
}

impl GetBlockResult {
    pub fn new(tag: BlockTag) -> Self {
        Self {
            shared_state: Arc::new(Mutex::new(SharedState::default())),
        }
    }
}

impl Future for GetBlockResult {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());

            Poll::Pending
        }
    }
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            completed: false,
            waker: None,
        }
    }
}
