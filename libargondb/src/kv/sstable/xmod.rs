mod compression;
mod descriptor;
mod index;
mod sstable_builder;

use descriptor::SSTableDescriptor;
pub use sstable_builder::SSTableBuilder;

struct SSTable {
    id: u64,
    info: SSTableDescriptor,
}
