use libargondb::{ClusteringKey, PartitionKey, TableMutation};

use super::{
    partition_cell::PartitionCell, partition_row::PartitionRowMut, Partition, PartitionMut,
    PartitionRow,
};

#[derive(Debug)]
pub struct PartitionBuilder {
    current_key: PartitionKey,
    current_partition: PartitionMut,
    row_builder: PartitionRowBuilder,
}

impl PartitionBuilder {
    pub fn new(mutation: &TableMutation) -> Self {
        let partition_key = mutation.key().partition_key().clone();
        let partition = PartitionMut::with_key(partition_key.clone());
        let row_builder = PartitionRowBuilder::new(mutation);

        Self {
            current_key: partition_key,
            current_partition: partition,
            row_builder,
        }
    }

    pub fn next(mut self, mutation: &TableMutation) -> Option<Partition> {
        let partition_key = mutation.key().partition_key();
        if partition_key < self.current_key {
            panic!("Keys must be in sorted order");
        }

        if partition_key == self.current_key {
            if let Some(row) = self.row_builder.next(mutation) {
                self.current_partition.push_row(row);
            }

            None
        } else {
            let row = PartitionRowBuilder::close(self.row_builder);
            self.current_partition.push_row(row);

            let partition: Partition = self.current_partition.into();

            self.current_partition = PartitionMut::with_key(partition_key);
            self.row_builder = PartitionRowBuilder::new(mutation);

            Some(partition)
        }
    }

    pub fn close(builder: Self) -> Partition {
        let row = PartitionRowBuilder::close(builder.row_builder);
        let mut partition = builder.current_partition;
        partition.push_row(row);

        partition.into()
    }
}

#[derive(Debug)]
struct PartitionRowBuilder {
    current_key: ClusteringKey,
    current_row: PartitionRowMut,
}

impl PartitionRowBuilder {
    fn new(mutation: &TableMutation) -> Self {
        let clustering_key = mutation.key().clustering_key().clone();
        let mut row = PartitionRowMut::with_key(clustering_key.clone());

        row.push_cell(PartitionCell::from(mutation.column_mutation()));

        Self {
            current_key: clustering_key,
            current_row: row,
        }
    }

    pub fn next(mut self, mutation: &TableMutation) -> Option<PartitionRow> {
        let clustering_key = mutation.key().clustering_key().clone();
        if clustering_key < self.current_key {
            panic!("Keys must be in sorted order");
        }

        if clustering_key == self.current_key {
            self.current_row
                .push_cell(PartitionCell::from(mutation.column_mutation()));

            None
        } else {
            let row: PartitionRow = self.current_row.into();

            self.current_row = PartitionRowMut::with_key(clustering_key);
            self.current_row
                .push_cell(PartitionCell::from(mutation.column_mutation()));

            Some(row)
        }
    }

    fn close(builder: Self) -> PartitionRow {
        builder.current_row.into()
    }
}
