mod index;
mod sstable_builder;
mod sstable_info;

pub use sstable_builder::SSTableBuilder;
use sstable_info::SSTableInfo;

struct SSTable {
    id: u64,
    info: SSTableInfo,
}
