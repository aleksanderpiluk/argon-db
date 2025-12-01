use std::sync::Arc;

use crate::{
    argonfs::{block_cache::BlockTag, core::BoxSSTableFormatReader},
    kv::KVSSTableBlockPtr,
};

pub struct FsReadRequest {
    pub block_tag: BlockTag,
    pub sstable_format_reader: Arc<BoxSSTableFormatReader>,
    pub sstable_ptr: KVSSTableBlockPtr,
}
