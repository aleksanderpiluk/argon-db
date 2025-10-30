use crate::kv::{
    memtable::Memtable,
    scan::{KVRangeScan, KVScanIterator, KVScannable},
};

pub struct Flusher {
    memtable: Memtable,
}

impl Flusher {
    pub fn new(memtable: Memtable) -> Self {
        // TODO: Assert memtable is in read-only state

        Self { memtable }
    }

    pub fn run(self) {
        // let mut sstable_builder: SSTableBuilder;

        todo!()
        // let params: RangeScanParams;
        // let mut iter = self.memtable.range_scan(params); // TODO: Full table scan

        // while let Some(mutation) = iter.next_mutation() {
        //     sstable_builder.write_mutation(mutation);
        // }
    }
}
