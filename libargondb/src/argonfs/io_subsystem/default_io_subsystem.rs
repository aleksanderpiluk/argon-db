use std::fs::File;

use async_executor::Executor;
use async_trait::async_trait;
use futures::future::BoxFuture;
use memmap::MmapOptions;

use crate::argonfs::io_subsystem::{IOFileReaderRequest, IOSubsystem};

pub struct DefaultIOSubsystem {
    executor: Executor<'static>,
}

impl DefaultIOSubsystem {
    pub fn new() -> DefaultIOSubsystem {
        todo!("spawn executor threadpool");
        Self {
            executor: Executor::new(),
        }
    }
}

#[async_trait]
impl IOSubsystem for DefaultIOSubsystem {
    async fn read(&self, request: IOFileReaderRequest) -> Result<Box<dyn AsRef<[u8]>>, ()> {
        let file = File::open(request.path).unwrap();
        let map = unsafe {
            MmapOptions::new()
                .offset(request.offset)
                .len(request.size)
                .map(&file)
        }
        .unwrap();

        Ok(Box::new(map))
    }

    fn pool_dispatch_task(&self, task: BoxFuture<'static, ()>) {
        self.executor.spawn(task).detach()
    }
}
