use crate::kv::KVSSTableBlockPtr;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlockTag {
    sstable_id: u64,
    block_ptr: KVSSTableBlockPtr,
}

impl BlockTag {
    pub fn new(sstable_id: u64, block_ptr: KVSSTableBlockPtr) -> Self {
        Self {
            sstable_id,
            block_ptr,
        }
    }
}
