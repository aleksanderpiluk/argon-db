use std::thread;

use super::{
    read::ReadRequest,
    read_thread::{ReadThreadContext, ReadThreadStrategy},
};

struct ReadThreadSyncStrategy;

impl ReadThreadStrategy for ReadThreadSyncStrategy {
    fn run(ctx: &ReadThreadContext) -> ! {
        loop {
            let request = ctx.read_queue.pop();
            match request {
                None => thread::yield_now(), // TODO: Consider change
                Some(request) => Self::process_request(request),
            }
        }
    }
}

impl ReadThreadSyncStrategy {
    fn process_request(request: ReadRequest) {
        // Step 1: Read with direct IO

        // Step 2: Decompress & validate block

        // Step 3: Write to page and mark as completed

        todo!()
    }
}
