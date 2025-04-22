use libargondb::PartitionKey;

use super::Stats;

#[derive(Debug)]
pub struct StatsBuilder {
    min_key: PartitionKey,
    max_key: PartitionKey,
}

impl StatsBuilder {
    pub fn new(min_key: &PartitionKey) -> Self {
        Self {
            min_key: min_key.clone(),
            max_key: min_key.clone(),
        }
    }

    pub fn close(mut self, max_key: &PartitionKey) -> Stats {
        self.max_key = max_key.clone();

        Stats {
            min_key: self.min_key,
            max_key: self.max_key,
        }
    }
}
