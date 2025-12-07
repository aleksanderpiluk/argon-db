use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;

use crate::{
    argonfs::block_cache::{self, BlockCache, BlockSharedGuard, BlockTag},
    kv::{
        KVRangeScan, KVRuntimeError, KVSSTableStats, KVSSTableSummaryIndex, KVScanIterator,
        KVScannable,
    },
    platform::io::ReadOnlyFileHandle,
};

pub struct ArgonfileSSTable {
    // file_handle: Box<dyn ReadOnlyFileHandle>,
    block_cache: Arc<BlockCache>,
    sstable_id: u64,
    summary_index: KVSSTableSummaryIndex,
    stats: KVSSTableStats,
}

impl ArgonfileSSTable {
    pub fn new(block_cache: Arc<BlockCache>) -> Self {
        todo!()
    }
}

#[async_trait]
impl KVScannable for ArgonfileSSTable {
    async fn range_scan(
        &self,
        scan: &KVRangeScan,
    ) -> Result<Box<dyn KVScanIterator>, KVRuntimeError> {
        // 1. Create "scan plan" by iterating through always loaded index and take blocks to scan through
        let blocks = self.summary_index.get_range_scan_blocks(scan);
        // 2. Create SSTableScanIter and return it - iterator makes proper scan
        let iter = KVSSTableScanIter::new(reader, blocks).await;

        let block_ptr = todo!();

        let block_guard = ReadBlockFuture::new(
            self.block_cache.clone(),
            BlockTag::new(self.sstable_id, block_ptr),
        )
        .await;
        todo!()
    }
}

struct ReadBlockFuture {
    block_cache: Arc<BlockCache>,
    block_tag: BlockTag,
}

impl ReadBlockFuture {
    fn new(block_cache: Arc<BlockCache>, block_tag: BlockTag) -> Self {
        todo!()
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

            todo!("dispatch read");
        }

        Poll::Pending
    }
}

// struct BlockReadyFuture {
//     block_cache: Arc<BlockCache>,
//     format_reader: Arc<BoxedArgonFsFormatReader>,
//     io_subsystem: Arc<IOSubsystem>,
//     block_tag: BlockTag,
//     ptr: KVSSTableBlockPtr,
// }

// impl BlockReadyFuture {
//     fn new(
//         block_cache: Arc<BlockCache>,
//         format_reader: Arc<BoxedArgonFsFormatReader>,
//         io_subsystem: Arc<IOSubsystem>,
//         block_tag: BlockTag,
//         ptr: KVSSTableBlockPtr,
//     ) -> Self {
//         Self {
//             block_cache,
//             format_reader,
//             io_subsystem,
//             block_tag,
//             ptr,
//         }
//     }
// }

// impl Future for BlockReadyFuture {
//     type Output = ();

//     fn poll(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         let block = self.block_cache.get_block(&self.block_tag, false);
//         let mut block = block.to_exclusive();

//         if block.is_loaded_block() {
//             return Poll::Ready(());
//         }

//         assert!(block.is_acquired());
//         let is_dispatched = block.is_read_dispatched();

//         block.add_waker(cx.waker().clone());

//         if !is_dispatched {
//             block.set_read_dispatched_flag();

//             let sstable_format_reader = self.format_reader.clone();
//             let block_tag = *block.block_tag();
//             let sstable_ptr = self.ptr;

//             let read_request = FsReadRequest {
//                 block_tag,
//                 sstable_format_reader,
//                 sstable_ptr,
//             };
//             self.io_subsystem.fs_read_request_queue().push(read_request);
//         }

//         Poll::Pending
//     }
// }
