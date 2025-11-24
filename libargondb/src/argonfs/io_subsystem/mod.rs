mod default_io_subsystem;

use std::{fs::File, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use futures::future::BoxFuture;

pub struct IOFileReaderRequest {
    pub path: PathBuf,
    pub offset: u64,
    pub size: usize,
}

#[async_trait]
pub trait IOSubsystem {
    async fn read(&self, request: IOFileReaderRequest) -> Result<Box<dyn AsRef<[u8]>>, ()>;

    fn pool_dispatch_task(&self, task: BoxFuture<'static, ()>);
}

pub type BoxIOSubsystem = Box<dyn IOSubsystem + Send + Sync>;
