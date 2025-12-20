use std::{fmt::Display, io::Write, sync::Arc};

use async_trait::async_trait;

use crate::{
    ArgonFsConfig,
    argonfs::{
        argon_fs_worker_pool::ArgonFsWorkerPool,
        argonfile_sstable::{ArgonfileSSTable, ArgonfileSSTableLoadError},
        block_cache::BlockCache,
        fs::{BoxFileSystem, FileSystemError},
        local_fs::FsFileSystem,
    },
    core::persistence::{PersistenceError, PersistenceLayer},
    kv::{KVInstanceStateSnapshot, KVScannable, KVTableId, ObjectId, schema::KVTableSchema},
    persistence::OrPersistenceError,
};

pub struct ArgonFs {
    block_cache: Arc<BlockCache>,
    filesystem: Arc<BoxFileSystem>,
    worker_pool: Arc<ArgonFsWorkerPool>,
}

impl ArgonFs {
    pub fn init(config: ArgonFsConfig) -> Result<Self, ArgonFsInitError> {
        let block_cache_config = config.to_block_cache_config();
        let block_cache: Arc<BlockCache> = Arc::new(BlockCache::new(block_cache_config));

        let filesystem: Arc<BoxFileSystem> = Arc::new(Box::new(FsFileSystem::new(
            config.fs_filesystem_config.clone(),
        )));

        let worker_pool = Arc::new(ArgonFsWorkerPool::new(1));

        Ok(Self {
            block_cache,
            filesystem,
            worker_pool,
        })
    }
}

#[async_trait]
impl PersistenceLayer for ArgonFs {
    async fn read_instance_snapshot(
        &self,
    ) -> Result<Option<KVInstanceStateSnapshot>, PersistenceError> {
        let file_ref = self
            .filesystem
            .get_state_snapshot_file_ref()
            .await
            .ok_or_persistence_error()?;

        match file_ref.open_read_only().await {
            Ok(mut reader) => {
                let object_id_generator_state = u64::from_le_bytes(
                    reader
                        .read(8)
                        .await
                        .ok_or_persistence_error()?
                        .as_ref()
                        .try_into()
                        .unwrap(),
                );

                Ok(Some(KVInstanceStateSnapshot {
                    object_id_generator_state,
                }))
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    println!("argonfs - instance snapshot file not found");
                    Ok(None)
                }
                _ => Err(PersistenceError(Box::new(e))),
            },
        }
    }

    async fn save_instance_snapshot(
        &self,
        snapshot: KVInstanceStateSnapshot,
    ) -> Result<(), PersistenceError> {
        let file_ref = self
            .filesystem
            .get_state_snapshot_file_ref()
            .await
            .ok_or_persistence_error()?;

        let mut writer = file_ref.open_write_only().await.ok_or_persistence_error()?;

        writer
            .write(&u64::to_le_bytes(snapshot.object_id_generator_state))
            .ok_or_persistence_error()?;
        writer.flush().ok_or_persistence_error()?;

        Ok(())
    }

    async fn scan_for_sstables(
        &self,
        table_id: &KVTableId,
        table_schema: &KVTableSchema,
    ) -> Result<Vec<Box<dyn KVScannable>>, PersistenceError> {
        let sstable_refs = self
            .filesystem
            .scan_table_catalog(table_id, table_schema)
            .await
            .ok_or_persistence_error()?;

        let mut sstables: Vec<Box<dyn KVScannable>> = vec![];
        for file_ref in sstable_refs {
            let argonfile_sstable = ArgonfileSSTable::load(
                table_schema.clone(),
                self.block_cache.clone(),
                self.worker_pool.clone(),
                file_ref,
            )
            .await
            .ok_or_persistence_error()?;
            sstables.push(Box::new(argonfile_sstable));
        }

        Ok(sstables)
    }

    async fn new_file_writer_for_sstable(
        &self,
        table_id: &KVTableId<'_>,
        sstable_id: ObjectId,
    ) -> Result<Box<dyn Write + Send + Sync + 'static>, PersistenceError> {
        let file_ref = self
            .filesystem
            .get_sstable_file_ref(table_id, sstable_id)
            .await
            .ok_or_persistence_error()?;

        Ok(Box::new(
            file_ref.open_write_only().await.ok_or_persistence_error()?,
        ))
    }

    async fn open_sstable(
        &self,
        table_id: &KVTableId,
        sstable_id: ObjectId,
        table_schema: &KVTableSchema,
    ) -> Result<Box<dyn KVScannable + 'static>, PersistenceError> {
        let file_ref = self
            .filesystem
            .get_sstable_file_ref(table_id, sstable_id)
            .await
            .ok_or_persistence_error()?;

        let argonfile_sstable = ArgonfileSSTable::load(
            table_schema.clone(),
            self.block_cache.clone(),
            self.worker_pool.clone(),
            file_ref,
        )
        .await
        .ok_or_persistence_error()?;

        Ok(Box::new(argonfile_sstable))
    }
}

#[derive(Debug)]
pub struct ArgonFsInitError {}

impl Display for ArgonFsInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for ArgonFsInitError {}

#[derive(Debug)]
pub enum ArgonFsError {
    ArgonfileLoadError(ArgonfileSSTableLoadError),
    FileSystemError(FileSystemError),
}

impl From<ArgonfileSSTableLoadError> for ArgonFsError {
    fn from(value: ArgonfileSSTableLoadError) -> Self {
        todo!()
    }
}

impl From<FileSystemError> for ArgonFsError {
    fn from(value: FileSystemError) -> Self {
        todo!()
    }
}
