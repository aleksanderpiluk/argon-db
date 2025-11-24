use std::{sync::Arc, task::Poll};

use async_trait::async_trait;
use futures::FutureExt;

use crate::{
    argonfs::{
        block_cache::{BlockCache, BlockSharedGuard, BlockTag},
        block_cache_buffer_allocator::BlockCacheAllocator,
        io_subsystem::BoxIOSubsystem,
        sstable_format_reader::{BoxSSTableFormatReader, SSTableFormatReader},
    },
    kv::{
        KVSSTableBlockPtr, KVSSTableDataBlockIter, KVSSTableReader, KVSSTableStats,
        KVSSTableSummaryIndex, KVScanIteratorItem,
    },
};

pub struct CachedSSTableReader {
    sstable_id: u64,
    block_cache: Arc<BlockCache>,
    io_subsystem: Arc<BoxIOSubsystem>,
    format_reader: Arc<BoxSSTableFormatReader>,
}

impl CachedSSTableReader {
    pub fn new(
        block_cache: Arc<BlockCache>,
        io_subsystem: Arc<BoxIOSubsystem>,
        format_reader: Arc<BoxSSTableFormatReader>,
    ) -> Self {
        Self {
            sstable_id: todo!(),
            block_cache,
            io_subsystem,
            format_reader,
        }
    }
}

#[async_trait]
impl KVSSTableReader for CachedSSTableReader {
    async fn read_stats_and_index(&self) -> (KVSSTableStats, KVSSTableSummaryIndex) {
        // As this function should be called only once to create KVSSTable instance, there's no need to cache it
        // self.inner_reader.read_stats_and_index().await
        todo!()
    }

    async fn read_data_block(
        &self,
        ptr: &KVSSTableBlockPtr,
    ) -> Box<dyn KVSSTableDataBlockIter + Send + Sync> {
        let cache_tag = BlockTag::new(self.sstable_id, *ptr);
        loop {
            let block: BlockSharedGuard = self.block_cache.get_block(&cache_tag, true);

            if block.is_loaded_block() {
                return self
                    .format_reader
                    .get_data_block_iter(block.to_boxed_view());
            }

            BlockReadyFuture::new(
                self.block_cache.clone(),
                self.io_subsystem.clone(),
                self.format_reader.clone(),
                cache_tag,
                *ptr,
            )
            .await;
        }
    }
}

struct CachedReaderIter {}

impl KVSSTableDataBlockIter for CachedReaderIter {
    fn next(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        todo!()
    }
}

struct BlockReadyFuture {
    block_cache: Arc<BlockCache>,
    io_subsystem: Arc<BoxIOSubsystem>,
    inner_reader: Arc<BoxSSTableFormatReader>,
    block_tag: BlockTag,
    ptr: KVSSTableBlockPtr,
}

impl BlockReadyFuture {
    fn new(
        block_cache: Arc<BlockCache>,
        io_subsystem: Arc<BoxIOSubsystem>,
        format_reader: Arc<BoxSSTableFormatReader>,
        block_tag: BlockTag,
        ptr: KVSSTableBlockPtr,
    ) -> Self {
        Self {
            block_cache,
            io_subsystem,
            inner_reader: format_reader,
            block_tag,
            ptr,
        }
    }
}

impl Future for BlockReadyFuture {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let block = self.block_cache.get_block(&self.block_tag, false);
        let mut block = block.to_exclusive();

        if block.is_loaded_block() {
            return Poll::Ready(());
        }

        assert!(block.is_acquired());
        let is_dispatched = block.is_read_dispatched();

        block.add_waker(cx.waker().clone());

        if !is_dispatched {
            block.set_read_dispatched_flag();

            let block_cache = self.block_cache.clone();
            let inner_reader = self.inner_reader.clone();
            let block_tag = *block.block_tag();
            let ptr = self.ptr;

            self.io_subsystem.pool_dispatch_task(
                async move {
                    let mut block_alloc = BlockCacheAllocator::new(block_cache.clone(), block_tag);

                    let block_size = inner_reader.load_data_block(ptr, &mut block_alloc).await;

                    let mut block = block_alloc.into_block();
                    let wakers = block.set_state_loaded_block(block_size);
                    for waker in wakers {
                        waker.wake();
                    }
                }
                .boxed(),
            );
        }

        Poll::Pending
    }
}
