use crate::{data_types::BlockTag, kv::sstable::compression::SSTableCompressionType};

pub struct SSTableDescriptor {
    summary: BlockTag,
    filter: BlockTag,
}
