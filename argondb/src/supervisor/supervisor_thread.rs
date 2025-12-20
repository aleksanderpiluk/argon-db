use std::thread::{self, JoinHandle};

pub fn run_supervisor_thread() -> JoinHandle<()> {
    let thread_handle = thread::spawn(supervisor_thread);

    thread_handle
}

fn supervisor_thread() {}
