mod partition_builder;
mod partition_cell;
mod partition_reader;
mod partition_row;
mod partition_writer;

pub use partition_builder::PartitionBuilder;
use partition_row::PartitionRow;
pub use partition_writer::PartitionWriter;

use libargondb::PartitionKey;

#[derive(Debug, Clone)]
pub struct Partition {
    key: PartitionKey,
    rows: Box<[PartitionRow]>,
}

impl Partition {
    pub fn key(&self) -> &PartitionKey {
        &self.key
    }

    pub fn into_key(self) -> PartitionKey {
        self.key
    }
}

#[derive(Debug, Clone)]
pub struct PartitionMut {
    key: PartitionKey,
    rows: Vec<PartitionRow>,
}

impl PartitionMut {
    fn with_key(partition_key: PartitionKey) -> Self {
        Self {
            key: partition_key,
            rows: vec![],
        }
    }

    fn push_row(&mut self, row: PartitionRow) {
        self.rows.push(row);
    }
}

impl Into<Partition> for PartitionMut {
    fn into(self) -> Partition {
        Partition {
            key: self.key,
            rows: self.rows.into_boxed_slice(),
        }
    }
}
