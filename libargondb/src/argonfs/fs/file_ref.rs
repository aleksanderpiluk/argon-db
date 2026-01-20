use std::io;

use super::file_handle::{ReadOnlyFileHandle, WriteOnlyFileHandle};
use async_trait::async_trait;

#[async_trait]
pub trait FileRef: Send + Sync {
    async fn open_read_only(&self) -> Result<Box<dyn ReadOnlyFileHandle>, io::Error>;

    async fn open_write_only(&self) -> Result<Box<dyn WriteOnlyFileHandle>, io::Error>;

    async fn remove(self: Box<Self>) -> Result<(), io::Error>;

    fn box_clone(&self) -> BoxFileRef;
}

pub type BoxFileRef = Box<dyn FileRef>;
