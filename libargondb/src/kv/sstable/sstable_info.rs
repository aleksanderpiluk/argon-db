use crate::data_types::BlockTag;

pub struct SSTableInfo {
    compression: CompressionType,
    min_row: Box<[u8]>,
    max_row: Box<[u8]>,
    summary: BlockTag,
    filter: BlockTag,
}

enum CompressionType {
    None,
}
