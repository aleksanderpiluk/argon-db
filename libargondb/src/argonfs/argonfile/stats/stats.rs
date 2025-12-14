use crate::kv::KVRangeScan;

pub struct Stats {
    pub bloom_filter: (),
    pub min_row: Box<[u8]>,
    pub max_row: Box<[u8]>,
}

impl Stats {
    pub fn is_range_scan_intersecting(&self, range_scan: &KVRangeScan) -> bool {
        todo!()
    }
}
