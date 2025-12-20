use std::{
    error::Error,
    fmt::Display,
    io::{self, Write},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use bytes::Buf;

use crate::{
    argonfs::{
        argon_fs_worker_pool::ArgonFsWorkerPool,
        argonfile::{Argonfile, ArgonfileDataBlockIter, ArgonfileReaderError, BlockPointer},
        block_cache::{BlockCache, BlockSharedGuard, BlockTag, BlockView, BlockWriter},
        fs::BoxFileRef,
    },
    kv::{
        KVRangeScan, KVRangeScanResult, KVRuntimeError, KVSSTableDataBlockIter, KVScanIterator,
        KVScanIteratorItem, KVScannable, KVTableSchema, primary_key::KVPrimaryKeySchema,
    },
};

pub struct ArgonfileSSTable {
    schema: KVTableSchema,
    argonfile: Arc<Argonfile>,
    block_cache: Arc<BlockCache>,
    worker_pool: Arc<ArgonFsWorkerPool>,
}

impl ArgonfileSSTable {
    pub async fn load(
        schema: KVTableSchema,
        block_cache: Arc<BlockCache>,
        worker_pool: Arc<ArgonFsWorkerPool>,
        file_ref: BoxFileRef,
    ) -> Result<Self, ArgonfileSSTableLoadError> {
        let argonfile = Arc::new(Argonfile::from_file_ref(file_ref).await?);

        Ok(Self {
            schema,
            argonfile,
            block_cache,
            worker_pool,
        })
    }
}

#[derive(Debug)]
pub enum ArgonfileSSTableLoadError {
    ArgonfileReaderError(ArgonfileReaderError),
    IOError(io::Error),
}

impl From<io::Error> for ArgonfileSSTableLoadError {
    fn from(value: io::Error) -> Self {
        todo!()
    }
}

impl From<ArgonfileReaderError> for ArgonfileSSTableLoadError {
    fn from(value: ArgonfileReaderError) -> Self {
        todo!()
    }
}

impl Error for ArgonfileSSTableLoadError {}

impl Display for ArgonfileSSTableLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArgonfileSSTableLoadError")
    }
}

#[async_trait]
impl KVScannable for ArgonfileSSTable {
    async fn range_scan(
        &self,
        range_scan: &KVRangeScan,
    ) -> Result<KVRangeScanResult, KVRuntimeError> {
        let pk_schema = KVPrimaryKeySchema::from_columns_schema(&self.schema);

        let is_intersecting = self
            .argonfile
            .stats
            .is_range_scan_intersecting(&pk_schema, range_scan);
        if !is_intersecting {
            return Ok(KVRangeScanResult::Empty);
        }

        let iter = RangeScanIterator::new(
            &pk_schema,
            self.block_cache.clone(),
            self.worker_pool.clone(),
            self.argonfile.clone(),
            range_scan,
        )
        .await;

        Ok(KVRangeScanResult::Iter(Box::new(iter)))
    }
}

struct RangeScanIterator<B: Buf> {
    block_cache: Arc<BlockCache>,
    block_ptrs: Vec<BlockPointer>,
    argonfile: Arc<Argonfile>,
    next_block_idx: usize,
    current_block_iter: Option<ArgonfileDataBlockIter<B>>,
    current_entry: Option<Box<dyn KVScanIteratorItem + Send + Sync>>,
    worker_pool: Arc<ArgonFsWorkerPool>,
}

impl RangeScanIterator<Box<BlockView>> {
    async fn new(
        schema: &KVPrimaryKeySchema,
        block_cache: Arc<BlockCache>,
        worker_pool: Arc<ArgonFsWorkerPool>,
        argonfile: Arc<Argonfile>,
        range_scan: &KVRangeScan,
    ) -> Self {
        let block_ptrs = argonfile
            .summary_index
            .get_blocks_for_range_scan(schema, range_scan);

        let mut this = Self {
            block_cache,
            block_ptrs,
            argonfile,
            next_block_idx: 0,
            current_block_iter: None,
            current_entry: None,
            worker_pool,
        };

        // Load first entry on initialization
        this.load_next_iter().await;
        this.load_next_entry().await;

        this
    }

    async fn load_next_iter(&mut self) {
        if let Some(block_ptr) = self.block_ptrs.get(self.next_block_idx) {
            self.next_block_idx += 1;

            let block_guard = ReadBlockFuture::new(
                self.block_cache.clone(),
                BlockTag::new(self.argonfile.sstable_id, *block_ptr),
                self.argonfile.clone(),
                self.worker_pool.clone(),
            )
            .await;

            let next_iter = ArgonfileDataBlockIter::new(block_guard.to_block_view());
            self.current_block_iter = Some(next_iter);
        } else {
            self.current_block_iter = None;
        }
    }

    async fn load_next_entry(&mut self) {
        loop {
            if let Some(iter) = &mut self.current_block_iter {
                if let Some(entry) = iter.next() {
                    self.current_entry = Some(entry);
                    break;
                } else {
                    self.load_next_iter().await;
                }
            } else {
                self.current_entry = None;
                break;
            }
        }
    }
}

#[async_trait]
impl KVScanIterator for RangeScanIterator<Box<BlockView>> {
    async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        let entry = std::mem::take(&mut self.current_entry);
        self.load_next_entry().await;
        entry
    }

    fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>> {
        if let Some(entry) = &self.current_entry {
            Some(entry)
        } else {
            None
        }
    }
}

struct ReadBlockFuture {
    block_cache: Arc<BlockCache>,
    block_tag: BlockTag,
    argonfile: Arc<Argonfile>,
    worker_pool: Arc<ArgonFsWorkerPool>,
}

impl ReadBlockFuture {
    fn new(
        block_cache: Arc<BlockCache>,
        block_tag: BlockTag,
        argonfile: Arc<Argonfile>,
        worker_pool: Arc<ArgonFsWorkerPool>,
    ) -> Self {
        Self {
            block_cache,
            block_tag,
            argonfile,
            worker_pool,
        }
    }
}

impl Future for ReadBlockFuture {
    type Output = BlockSharedGuard;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let guard = self.block_cache.get_block(&self.block_tag, true);

        if guard.is_loaded_block() {
            return Poll::Ready(guard);
        }

        let mut guard = guard.to_exclusive();

        // TODO: Check
        assert!(guard.is_acquired());
        let is_dispatched = guard.is_read_dispatched();

        guard.add_waker(cx.waker().clone());

        if !is_dispatched {
            guard.set_read_dispatched_flag();
            drop(guard);

            let argonfile = self.argonfile.clone();
            let block_cache = self.block_cache.clone();
            let block_ptr = self.block_tag.block_ptr.clone();
            let block_tag = self.block_tag.clone();
            self.worker_pool
                .spawn(async move {
                    let block = argonfile.read_block(&block_ptr).await.unwrap();

                    let guard = block_cache.get_block(&block_tag, false);
                    let mut guard = guard.to_exclusive();

                    let is_loaded = guard.is_loaded_block();
                    let is_read_dispatched = guard.is_read_dispatched();
                    assert!(is_loaded == false);
                    assert!(is_read_dispatched == true);

                    let block_size = block.data.len();
                    block_cache.expand_block(&mut guard, block_size);

                    let mut writer = BlockWriter::new(guard);
                    writer.write_all(&block.data).unwrap();

                    let mut guard = writer.into_guard();
                    let wakers = guard.set_state_loaded_block(block_size);

                    drop(guard);
                    for waker in wakers {
                        waker.wake();
                    }
                })
                .detach();
        }

        Poll::Pending
    }
}
