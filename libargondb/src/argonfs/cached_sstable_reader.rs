// use std::{sync::Arc, task::Poll};

// use async_trait::async_trait;

// use super::core::BoxedArgonFsFormatReader;
// use crate::{
//     argonfs::{
//         block_cache::{BlockCache, BlockSharedGuard, BlockTag},
//         io_subsystem::{FsReadRequest, IOSubsystem},
//     },
//     kv::{
//         KVSSTableBlockPtr, KVSSTableDataBlockIter, KVSSTableReader, KVSSTableStats,
//         KVSSTableSummaryIndex, KVScanIteratorItem,
//     },
// };

// pub struct CachedSSTableReader {
//     sstable_id: u64,
//     io_subsystem: Arc<IOSubsystem>,
//     format_reader: Arc<BoxedArgonFsFormatReader>,
// }

// impl CachedSSTableReader {
//     pub fn new(
//         block_cache: Arc<BlockCache>,
//         io_subsystem: Arc<IOSubsystem>,
//         format_reader: Arc<BoxedArgonFsFormatReader>,
//     ) -> Self {
//         Self {
//             sstable_id: todo!(),
//             block_cache,
//             io_subsystem,
//             format_reader,
//         }
//     }
// }

// #[async_trait]
// impl KVSSTableReader for CachedSSTableReader {
//     async fn read_stats_and_index(&self) -> (KVSSTableStats, KVSSTableSummaryIndex) {
//         // As this function should be called only once to create KVSSTable instance, there's no need to cache it

//         let stats = self.format_reader.read_kv_sstable_stats();
//         let summary_index = self.format_reader.read_kv_sstable_summary_index();

//         (stats, summary_index)
//     }

//     async fn read_data_block(
//         &self,
//         ptr: &KVSSTableBlockPtr,
//     ) -> Box<dyn KVSSTableDataBlockIter + Send + Sync> {
//         let cache_tag = BlockTag::new(self.sstable_id, *ptr);
//         loop {
//             let block: BlockSharedGuard = self.block_cache.get_block(&cache_tag, true);

//             if block.is_loaded_block() {
//                 return self
//                     .format_reader
//                     .get_data_block_iter(block.to_boxed_view());
//             }

//             BlockReadyFuture::new(
//                 self.block_cache.clone(),
//                 self.format_reader.clone(),
//                 self.io_subsystem.clone(),
//                 cache_tag,
//                 *ptr,
//             )
//             .await;
//         }
//     }
// }

// struct CachedReaderIter {}

// impl KVSSTableDataBlockIter for CachedReaderIter {
//     fn next(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
//         todo!()
//     }
// }

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
