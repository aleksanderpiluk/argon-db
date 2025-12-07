use std::sync::Arc;

use crate::{
    argonfs::{block_cache::BlockTag, core::BoxedArgonFsFormatReader},
    kv::KVSSTableBlockPtr,
};

pub struct FsReadRequest {
    pub block_tag: BlockTag,
    pub sstable_format_reader: Arc<BoxedArgonFsFormatReader>,
    pub sstable_ptr: KVSSTableBlockPtr,
}
