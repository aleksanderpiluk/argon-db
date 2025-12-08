use async_io::block_on;
use futures::future;
use std::{ops::Deref, sync::Arc, thread};

use async_executor::Executor;

pub struct ArgonFsWorkerPool {
    executor: Arc<Executor<'static>>,
}

impl ArgonFsWorkerPool {
    pub fn new(num_threads: usize) -> Self {
        let executor = Arc::new(Executor::new());

        for _ in 0..num_threads {
            let ex = executor.clone();
            thread::spawn(move || block_on(ex.run(future::pending::<()>())));
        }

        Self { executor }
    }
}

impl Deref for ArgonFsWorkerPool {
    type Target = Executor<'static>;

    fn deref(&self) -> &Self::Target {
        &self.executor
    }
}
