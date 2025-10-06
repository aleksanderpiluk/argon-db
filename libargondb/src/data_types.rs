pub struct BlockTag(u64, u64, u32);

impl BlockTag {
    fn new(sstable_id: u64, block_ptr: u64, disk_size: u32) -> Self {
        Self(sstable_id, block_ptr, disk_size)
    }
}
