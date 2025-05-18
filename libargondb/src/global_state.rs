use crate::io;

struct GlobalState {
    read_queue: io::read::ReadQueue,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            read_queue: io::read::ReadQueue::new(32),
        }
    }
}
