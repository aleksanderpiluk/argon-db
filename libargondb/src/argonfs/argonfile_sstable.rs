use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use bytes::Buf;
use thiserror::Error;

use crate::{
    argonfs::{
        argon_fs_worker_pool::ArgonFsWorkerPool,
        argonfile::{Argonfile, ArgonfileDataBlockIter, ArgonfileReader, ArgonfileReaderError},
        block_cache::{BlockCache, BlockSharedGuard, BlockTag, BlockView},
    },
    kv::{
        KVRangeScan, KVRuntimeError, KVSSTableBlockPtr, KVSSTableDataBlockIter, KVScanIterator,
        KVScanIteratorItem, KVScannable,
    },
    platform::io::{BoxFileRef, FileRef},
};

pub struct ArgonfileSSTable {
    block_cache: Arc<BlockCache>,
    file_ref: BoxFileRef,
    argonfile: Argonfile,
    worker_pool: Arc<ArgonFsWorkerPool>,
}

impl ArgonfileSSTable {
    pub async fn load(
        block_cache: Arc<BlockCache>,
        worker_pool: Arc<ArgonFsWorkerPool>,
        file_ref: BoxFileRef,
    ) -> Result<Self, ArgonfileSSTableLoadError> {
        let file_handle = file_ref.open_read_only().await?;
        let mut reader = ArgonfileReader::new(file_handle);

        let trailer = reader.read_trailer().await?;
        let sstable_id = trailer.sstable_id;

        let summary_index = reader.read_summary_index().await?;
        let stats = reader.read_stats().await?;

        let argonfile = todo!();

        Ok(Self {
            block_cache,
            file_ref,
            argonfile,
            worker_pool,
        })
    }
}

#[derive(Error, Debug)]
pub enum ArgonfileSSTableLoadError {
    #[error("argonfile reader error - {0}")]
    ArgonfileReaderError(#[from] ArgonfileReaderError),

    #[error("io error - {0}")]
    IOError(#[from] io::Error),
}

#[async_trait]
impl KVScannable for ArgonfileSSTable {
    async fn range_scan(
        &self,
        range_scan: &KVRangeScan,
    ) -> Result<Box<dyn KVScanIterator + Send + Sync>, KVRuntimeError> {
        let is_intersecting = self.argonfile.stats.is_range_scan_intersecting(range_scan);
        if !is_intersecting {
            todo!("RETURN EMPTY");
        }

        let block_ptrs = self
            .argonfile
            .summary_index
            .get_blocks_for_range_scan(range_scan);

        // 2. Create SSTableScanIter and return it - iterator makes proper scan
        let iter = ScanIterator::new(
            self.block_cache.clone(),
            blocks,
            self.sstable_id,
            self.file_ref.box_clone(),
            self.worker_pool.clone(),
        )
        .await;

        Ok(Box::new(iter))
    }
}

pub struct ScanIterator<B: Buf> {
    block_cache: Arc<BlockCache>,
    blocks: Vec<KVSSTableBlockPtr>,
    sstable_id: u64,
    file_ref: BoxFileRef,
    next_block_idx: usize,
    current_block_iter: Option<ArgonfileDataBlockIter<B>>,
    current_entry: Option<Box<dyn KVScanIteratorItem + Send + Sync>>,
    worker_pool: Arc<ArgonFsWorkerPool>,
}

impl ScanIterator<Box<BlockView>> {
    async fn new(
        block_cache: Arc<BlockCache>,
        blocks: Vec<KVSSTableBlockPtr>,
        sstable_id: u64,
        file_ref: BoxFileRef,
        worker_pool: Arc<ArgonFsWorkerPool>,
    ) -> Self {
        let mut this = Self {
            block_cache,
            blocks,
            sstable_id,
            file_ref,
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
        if let Some(block_ptr) = self.blocks.get(self.next_block_idx) {
            self.next_block_idx += 1;

            let block_guard = ReadBlockFuture::new(
                self.block_cache.clone(),
                BlockTag::new(self.sstable_id, *block_ptr),
                self.file_ref.box_clone(),
                self.worker_pool.clone(),
            )
            .await;

            let next_iter = ArgonfileDataBlockIter::new(block_guard.to_boxed_view());
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
impl KVScanIterator for ScanIterator<Box<BlockView>> {
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
    file_ref: BoxFileRef,
    worker_pool: Arc<ArgonFsWorkerPool>,
}

impl ReadBlockFuture {
    fn new(
        block_cache: Arc<BlockCache>,
        block_tag: BlockTag,
        file_ref: Box<dyn FileRef>,
        worker_pool: Arc<ArgonFsWorkerPool>,
    ) -> Self {
        Self {
            block_cache,
            block_tag,
            file_ref,
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

            let file_ref = self.file_ref.box_clone();
            let block_ptr = self.block_tag.block_ptr.clone();
            self.worker_pool.spawn(async move {
                let file_handle = file_ref.open_read_only().await.unwrap();
                let mut reader = ArgonfileReader::new(file_handle);

                reader.read_block(&block_ptr.into()).await;
            });
        }

        Poll::Pending
    }
}
