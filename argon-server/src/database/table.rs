use bytes::Bytes;
use dashmap::DashSet;

// use super::memstore::Memstore;

pub struct Table {
    // memstore: Memstore,
    // ss_tables: DashSet<SSTablePtr>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableId(Bytes);

pub struct TableName(Bytes);
