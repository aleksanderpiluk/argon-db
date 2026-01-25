use crate::{argonfs::argonfile::BlockPointer, kv::ObjectId};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlockTag {
    pub sstable_id: ObjectId,
    pub block_ptr: BlockPointer,
}

impl BlockTag {
    pub fn new(sstable_id: ObjectId, block_ptr: BlockPointer) -> Self {
        Self {
            sstable_id,
            block_ptr,
        }
    }
}
