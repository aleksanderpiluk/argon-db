use std::io;

use async_trait::async_trait;

use crate::platform::io::ReadOnlyFileHandle;

#[async_trait]
pub trait FileRef: Send + Sync {
    async fn open_read_only(&self) -> Result<Box<dyn ReadOnlyFileHandle>, io::Error>;

    fn box_clone(&self) -> BoxFileRef;
}

pub type BoxFileRef = Box<dyn FileRef>;
