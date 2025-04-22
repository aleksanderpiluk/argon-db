mod stats_builder;

use libargondb::PartitionKey;
pub use stats_builder::StatsBuilder;

pub struct Stats {
    min_key: PartitionKey,
    max_key: PartitionKey,
}

impl Stats {
    pub fn min_key(&self) -> &PartitionKey {
        &self.min_key
    }

    pub fn max_key(&self) -> &PartitionKey {
        &self.max_key
    }
}
