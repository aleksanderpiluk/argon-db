use bytes::Buf;

use crate::{
    argonfs::core::BufferAllocator,
    kv::{KVSSTableBlockPtr, KVSSTableDataBlockIter, KVSSTableStats, KVSSTableSummaryIndex},
};

pub trait ArgonFsFormatReader {
    fn read_kv_sstable_stats(&self) -> Result<KVSSTableStats, ArgonFsFormatReaderError>;

    fn read_kv_sstable_summary_index(
        &self,
    ) -> Result<KVSSTableSummaryIndex, ArgonFsFormatReaderError>;

    fn load_data_block(&self, ptr: KVSSTableBlockPtr, alloc_fn: &mut dyn BufferAllocator) -> usize;

    fn get_data_block_iter(
        &self,
        data_block: Box<dyn Buf>,
    ) -> Box<dyn KVSSTableDataBlockIter + Send + Sync>;
}

pub type BoxedArgonFsFormatReader = Box<dyn ArgonFsFormatReader + Send + Sync>;

pub enum ArgonFsFormatReaderError {}
