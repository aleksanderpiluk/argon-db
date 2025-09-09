use std::{fs::File, hint, mem::replace, os::fd::AsRawFd};

use crossbeam::queue::ArrayQueue;
use io_uring::{IoUring, opcode, types};

#[derive(Debug)]
struct ReadRequest {
    file: File,
    offset: u64,
    buffer: String,
}

struct ReadExecutor {
    ring: IoUring,
    requests_pool: ReadRequestsPool,
    queue: ArrayQueue<ReadRequest>,
}

impl ReadExecutor {
    fn run(mut self) {
        loop {
            self.process_completions();

            self.process_submissions();

            // TODO: Better handling of busy-wait loop based on submissions/completions
            hint::spin_loop();
        }
    }

    fn process_completions(&mut self) {
        let mut completion = self.ring.completion();
        completion.sync();

        while let Some(entry) = completion.next() {
            let id = entry.user_data();

            let request = self.requests_pool.take_request(id as _).unwrap();
            todo!("process request");
        }
    }

    fn process_submissions(&mut self) {
        while let Some(request) = self.queue.pop() {
            match self.try_dispatch_request(request) {
                Ok(_) => {}
                Err(err) => {
                    match err {
                        ReadRequestDispatchError::RingFull(request) => {
                            //
                            self.ring.submit_and_wait(1).unwrap();
                            self.process_completions();
                            self.try_dispatch_request(request)
                                .expect("failed to dispatch request");
                        }
                    }
                }
            }
        }
    }

    fn try_dispatch_request(
        &mut self,
        request: ReadRequest,
    ) -> Result<(), ReadRequestDispatchError> {
        let entry = opcode::Read::new(
            types::Fd(request.file.as_raw_fd()),
            request.buffer.as_ptr().cast_mut(),
            request.buffer.len() as _,
        )
        .offset(request.offset)
        .build();

        let id = match self.requests_pool.put_request(request) {
            Ok(id) => id,
            Err(request) => return Err(ReadRequestDispatchError::RingFull(request)),
        };
        let entry = entry.user_data(id as _);

        unsafe {
            self.ring
                .submission()
                .push(&entry)
                .expect("submission queue is full");
        }

        self.ring.submit().unwrap();

        Ok(())
    }
}

struct ReadRequestsPool {
    items: Box<[ReadRequestsPoolItem]>,
    next_free: Option<usize>,
}

#[derive(Debug)]
enum ReadRequestDispatchError {
    RingFull(ReadRequest),
}

impl ReadRequestsPool {
    fn new(size: usize) -> Self {
        assert!(size > 0);

        let items = (0..size)
            .into_iter()
            .map(|i| ReadRequestsPoolItem::Free {
                next_free: if i + 1 < size { Some(i + 1) } else { None },
            })
            .collect::<Vec<ReadRequestsPoolItem>>()
            .into_boxed_slice();
        let next_free = Some(0);

        Self { items, next_free }
    }

    fn put_request(&mut self, request: ReadRequest) -> Result<usize, ReadRequest> {
        let free_item = match self.next_free {
            Some(next_free) => next_free,
            None => return Err(request),
        };

        match self.items[free_item] {
            ReadRequestsPoolItem::Free { next_free } => self.next_free = next_free,
            ReadRequestsPoolItem::Occupied { request: _ } => panic!("item not in free state"),
        };

        self.items[free_item] = ReadRequestsPoolItem::Occupied { request };

        Ok(free_item)
    }

    fn take_request(&mut self, id: usize) -> Result<ReadRequest, ()> {
        if let ReadRequestsPoolItem::Free { next_free: _ } = &self.items[id] {
            panic!("item in free state");
        }

        let item = replace(
            &mut self.items[id],
            ReadRequestsPoolItem::Free {
                next_free: self.next_free,
            },
        );
        self.next_free = Some(id);

        let ReadRequestsPoolItem::Occupied { request } = item else {
            panic!("item not in occupied state")
        };

        Ok(request)
    }
}

enum ReadRequestsPoolItem {
    Free { next_free: Option<usize> },
    Occupied { request: ReadRequest },
}
