use crate::kv::KVRangeScan;

pub struct SummaryIndex {}

impl SummaryIndex {
    pub fn get_blocks_for_range_scan(&self, range_scan: &KVRangeScan) -> Vec<()> {
        todo!()
    }
}
