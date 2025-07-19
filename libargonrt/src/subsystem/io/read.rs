use std::thread;

use crossbeam::queue::ArrayQueue;
use io_uring::{IoUring, opcode, types};
use slab::Slab;

use crate::subsystem::io::block_cache::page::Page;

static read_queue: ArrayQueue<ReadRequest> = ArrayQueue::new(32);
static read_slab: Slab<ReadRequest> = Slab::new();

struct Read;

impl Read {
    fn read(request: ReadRequest) {
        read_queue.push(request).unwrap();
    }
}

fn read_init() {
    let mut ring = IoUring::new(8).unwrap();

    thread::spawn(|| {
        loop {
            // TODO: It NEEDS ACCESS to page and it needs to stay in memory, so we need page ref here and exclusive write access(?)
            while let Some(request) = read_queue.pop() {
                // TODO: Schedule request
                let op = request.get_op();
                let idx = read_slab.insert(request);
                let entry = op.build().user_data(user_data);

                unsafe {
                    ring.submission()
                        .push(entry)
                        .expect("submission queue is full");
                }

                ring.submit().unwrap();
            }

            if let Some(cqe) = ring.completion().next() {
                let request = read_slab.remove(cqe.user_data());
                todo!()
                // TODO: Call wakers
            }
        }
    });
}

struct ReadRequest {
    page: Page,
}

impl ReadRequest {
    fn get_op(&self) -> opcode::Read {
        opcode::Read::new(fd, buf, len).offset(offset)
    }
}
