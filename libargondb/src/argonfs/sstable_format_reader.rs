use async_trait::async_trait;
use bytes::Buf;

use crate::{
    argonfs::buffer_allocator::BufferAllocator,
    kv::{KVSSTableBlockPtr, KVSSTableDataBlockIter},
};

#[async_trait]
pub trait SSTableFormatReader {
    async fn load_data_block(
        &self,
        ptr: KVSSTableBlockPtr,
        // io: Arc<BoxIOSubsystem>,
        alloc_fn: &mut dyn BufferAllocator,
    ) -> usize;

    fn get_data_block_iter(
        &self,
        data_block: Box<dyn Buf>,
    ) -> Box<dyn KVSSTableDataBlockIter + Send + Sync>;
}

pub type BoxSSTableFormatReader = Box<dyn SSTableFormatReader + Send + Sync>;
