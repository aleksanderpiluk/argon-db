use std::io::Write;

use crate::{
    core::persistence::PersistenceError,
    kv::{KVInstanceStateSnapshot, KVSSTable, KVScannable, KVTableId, KVTableSchema, ObjectId},
};
use async_trait::async_trait;

#[async_trait]
pub trait PersistenceLayer {
    async fn read_instance_snapshot(
        &self,
    ) -> Result<Option<KVInstanceStateSnapshot>, PersistenceError>;

    async fn save_instance_snapshot(
        &self,
        snapshot: KVInstanceStateSnapshot,
    ) -> Result<(), PersistenceError>;

    async fn scan_for_sstables(
        &self,
        table_id: &KVTableId,
        table_schema: &KVTableSchema,
    ) -> Result<Vec<Box<dyn KVSSTable + 'static>>, PersistenceError>;

    async fn new_file_writer_for_sstable(
        &self,
        table_id: &KVTableId<'_>,
        sstable_id: ObjectId,
    ) -> Result<Box<dyn Write + Send + Sync + 'static>, PersistenceError>;

    async fn open_sstable(
        &self,
        table_id: &KVTableId,
        sstable_id: ObjectId,
        table_schema: &KVTableSchema,
    ) -> Result<Box<dyn KVSSTable + 'static>, PersistenceError>;

    async fn remove_compacted_sstables(
        &self,
        table_id: &KVTableId,
        sstable_ids: Vec<ObjectId>,
    ) -> Result<(), PersistenceError>;
}

pub type BoxPersistenceLayer = Box<dyn PersistenceLayer + Send + Sync>;
