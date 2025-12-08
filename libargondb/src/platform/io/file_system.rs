use std::io;

use async_trait::async_trait;
use thiserror::Error;

use crate::platform::io::BoxFileRef;

use super::TableCatalogRef;

#[async_trait]
pub trait FileSystem {
    async fn scan_table_catalogs(&self) -> Result<Vec<Box<dyn TableCatalogRef>>, FileSystemError>;

    async fn scan_table_catalog(
        &self,
        table_name: &str,
    ) -> Result<Vec<BoxFileRef>, FileSystemError>;
}

pub type BoxFileSystem = Box<dyn FileSystem>;

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("io error - {0}")]
    IOError(#[from] io::Error),
}
