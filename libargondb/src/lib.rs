use anyhow::{Context, Ok, Result, anyhow};

#[derive(Debug, Clone)]
pub struct TableMutation(PrimaryKey, ColumnMutation);

impl TableMutation {
    pub fn key(&self) -> PrimaryKey {
        todo!()
    }

    pub fn column_mutation(&self) -> ColumnMutation {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ColumnMutation {
    Put {
        column: ColumnId,
        timestamp: Timestamp,
        value: CellValue,
    },
    Delete {
        column: ColumnId,
        timestamp: Timestamp,
    },
    GroupDelete {
        columns: Box<[ColumnId]>,
        timestamp: Timestamp,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnId(u16);

impl From<u16> for ColumnId {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Into<u16> for ColumnId {
    fn into(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Timestamp(u64);

impl From<u64> for Timestamp {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Into<u64> for Timestamp {
    fn into(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct CellValue(Box<[u8]>);

impl CellValue {
    pub fn inner(&self) -> &Box<[u8]> {
        &self.0
    }
}

impl From<Box<[u8]>> for CellValue {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrimaryKey;

impl PrimaryKey {
    pub fn partition_key(&self) -> PartitionKey {
        todo!()
    }

    pub fn clustering_key(&self) -> ClusteringKey {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PartitionKey(Box<[u8]>);

impl PartitionKey {
    pub fn len(&self) -> u16 {
        self.0.len() as u16
    }
}

impl Into<Box<[u8]>> for PartitionKey {
    fn into(self) -> Box<[u8]> {
        self.0
    }
}

impl TryFrom<Box<[u8]>> for PartitionKey {
    type Error = anyhow::Error;

    fn try_from(value: Box<[u8]>) -> std::result::Result<Self, Self::Error> {
        let key_size = value.len();
        if key_size > u16::MAX as usize {
            return Err(anyhow!("Key exceeds max allowed size for partition key"));
        }

        Ok(Self(value))
    }
}

impl AsRef<Box<[u8]>> for PartitionKey {
    fn as_ref(&self) -> &Box<[u8]> {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClusteringKey(Box<[u8]>);

impl ClusteringKey {
    pub fn inner(&self) -> &Box<[u8]> {
        &self.0
    }
}

impl Into<Box<[u8]>> for ClusteringKey {
    fn into(self) -> Box<[u8]> {
        self.0
    }
}

impl From<Box<[u8]>> for ClusteringKey {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
    }
}
