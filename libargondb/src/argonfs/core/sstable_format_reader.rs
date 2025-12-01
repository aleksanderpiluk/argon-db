use bytes::Buf;

use crate::{
    argonfs::core::BufferAllocator,
    kv::{KVSSTableBlockPtr, KVSSTableDataBlockIter},
};

pub trait SSTableFormatReader {
    fn load_data_block(&self, ptr: KVSSTableBlockPtr, alloc_fn: &mut dyn BufferAllocator) -> usize;

    fn get_data_block_iter(
        &self,
        data_block: Box<dyn Buf>,
    ) -> Box<dyn KVSSTableDataBlockIter + Send + Sync>;
}

pub type BoxSSTableFormatReader = Box<dyn SSTableFormatReader + Send + Sync>;

pub enum SSTableFormatReaderError {}
