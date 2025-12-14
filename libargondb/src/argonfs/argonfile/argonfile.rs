use super::{stats::Stats, summary::SummaryIndex};

pub struct Argonfile {
    pub sstable_id: u64,
    pub summary_index: SummaryIndex,
    pub stats: Stats,
}
