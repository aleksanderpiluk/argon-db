use std::io;

use async_trait::async_trait;
use thiserror::Error;

use super::BoxFileRef;
use crate::kv::{Id, KVTableSchema, ObjectId};

#[async_trait]
pub trait FileSystem {
    async fn scan_table_catalog(
        &self,
        table_id: &Id,
        table_schema: &KVTableSchema,
    ) -> Result<Vec<BoxFileRef>, FileSystemError>;

    async fn get_sstable_file_ref(
        &self,
        table_id: &Id,
        sstable_id: ObjectId,
    ) -> Result<BoxFileRef, FileSystemError>;

    async fn get_state_snapshot_file_ref(&self) -> Result<BoxFileRef, FileSystemError>;
}

pub type BoxFileSystem = Box<dyn FileSystem + Send + Sync>;

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("io error - {0}")]
    IOError(#[from] io::Error),
}
