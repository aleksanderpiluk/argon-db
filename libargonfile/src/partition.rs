use crate::partition_header::PartitionHeader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partition {
    header: PartitionHeader,
}

pub struct PartitionReader {}

pub struct PartitionWriter;

impl PartitionWriter {}
