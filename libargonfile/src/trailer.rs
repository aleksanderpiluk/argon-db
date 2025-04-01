use crate::block_pointer::BlockPointer;

pub struct Trailer {
    compression_coded: u16,
    min_key_size: u16,
    max_key_size: u16,
    summary_block: BlockPointer,
    filter_block: BlockPointer,
}
