use std::thread::JoinHandle;

struct ThreadVisor {
    handlers: Vec<ThreadVisorJoinHandle>,
}

impl ThreadVisor {
    fn spawn_thread() {
        todo!()
    }

    fn check_threads(&mut self) {
        for handler in &self.handlers {
            ThreadVisor::check_handler(handler);
        }
    }
}

impl ThreadVisor {
    fn check_handler(handler: &ThreadVisorJoinHandle) {
        if handler.is_finished() {}
    }
}

type ThreadVisorJoinHandle = JoinHandle<()>;
