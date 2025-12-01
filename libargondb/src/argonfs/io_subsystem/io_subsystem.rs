use std::sync::Arc;

use crate::argonfs::io_subsystem::{
    BoxedPlatformIOAdapter, default_platform_io::DefaultPlatformIo,
    fs_read_pool::FsReadRequestQueue,
};

pub struct IOSubsystem {
    platform_io_adapter: Arc<BoxedPlatformIOAdapter>,
    fs_read_request_queue: Arc<FsReadRequestQueue>,
}

impl IOSubsystem {
    pub fn init() -> Result<Self, IOSubsystemInitError> {
        let platform_io_adapter: Arc<BoxedPlatformIOAdapter> =
            Arc::new(Box::new(DefaultPlatformIo::new()));
        let fs_read_request_queue = Arc::new(FsReadRequestQueue::new());

        Ok(Self {
            platform_io_adapter,
            fs_read_request_queue,
        })
    }

    pub fn platform_io_adapter(&self) -> &BoxedPlatformIOAdapter {
        &self.platform_io_adapter
    }

    pub fn fs_read_request_queue(&self) -> &FsReadRequestQueue {
        &self.fs_read_request_queue
    }
}

#[derive(Debug)]
pub enum IOSubsystemInitError {}
