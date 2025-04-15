use std::io::Write;

use anyhow::{anyhow, Ok};
use libargondb::{CellValue, ColumnId, ColumnMutation, Timestamp};

use crate::{
    pointer::Pointer,
    shared::{Reader, Writer},
};

#[derive(Debug, Clone)]
pub struct PartitionCell(ColumnMutation);

impl From<ColumnMutation> for PartitionCell {
    fn from(value: ColumnMutation) -> Self {
        Self(value)
    }
}

pub struct PartitionCellWriter;

impl Writer<PartitionCell> for PartitionCellWriter {
    fn try_write<W: std::io::Write>(
        writer: &mut crate::shared::PositionedWriter<W>,
        cell: &PartitionCell,
    ) -> anyhow::Result<crate::pointer::Pointer> {
        let offset = writer.get_position();
        let mut size: usize = 0;

        let cell_type = CellType::for_column_mutation(&cell.0);
        size += writer.write(&u16::to_be_bytes(cell_type.into()))?;
        match cell.0.clone() {
            ColumnMutation::Put {
                column,
                timestamp,
                value,
            } => {
                size += writer.write(&u16::to_be_bytes(column.into()))?;
                size += writer.write(&u64::to_be_bytes(timestamp.into()))?;
                size += writer.write(&u16::to_be_bytes(value.inner().len() as u16))?;
                size += writer.write(&value.inner())?;
            }
            ColumnMutation::Delete { column, timestamp } => {
                size += writer.write(&u16::to_be_bytes(column.into()))?;
                size += writer.write(&u64::to_be_bytes(timestamp.into()))?;
            }
            ColumnMutation::GroupDelete { columns, timestamp } => {
                size += writer.write(&u16::to_be_bytes(columns.len() as u16))?;
                size += writer.write(&u64::to_be_bytes(timestamp.into()))?;
                for column in columns {
                    size += writer.write(&u16::to_be_bytes(column.into()))?;
                }
            }
        }

        Ok(Pointer::new(offset, size))
    }
}

pub struct PartitionCellReader;

impl Reader<PartitionCell> for PartitionCellReader {
    fn try_read<R: std::io::Read>(reader: &mut R) -> anyhow::Result<PartitionCell> {
        let mut buf = vec![0u8; 4];
        reader.read_exact(&mut buf)?;

        let cell_type = CellType::from(u16::from_be_bytes(buf[0..2].try_into().unwrap()));

        match cell_type {
            CellType::PUT => {
                let column_id = u16::from_be_bytes(buf[2..4].try_into().unwrap());

                let mut buf = vec![0u8; 10];
                reader.read_exact(&mut buf)?;
                let timestamp = u64::from_be_bytes(buf[0..8].try_into().unwrap());
                let value_size = u16::from_be_bytes(buf[8..10].try_into().unwrap());

                let mut buf = vec![0u8; value_size as usize];
                reader.read_exact(&mut buf);

                Ok(PartitionCell(ColumnMutation::Put {
                    column: ColumnId::from(column_id),
                    timestamp: Timestamp::from(timestamp),
                    value: CellValue::from(buf.into_boxed_slice()),
                }))
            }
            CellType::DELETE => {
                let column_id = u16::from_be_bytes(buf[2..4].try_into().unwrap());

                let mut buf = vec![0u8; 8];
                reader.read_exact(&mut buf)?;
                let timestamp = u64::from_be_bytes(buf[0..8].try_into().unwrap());
                Ok(PartitionCell(ColumnMutation::Delete {
                    column: ColumnId::from(column_id),
                    timestamp: Timestamp::from(timestamp),
                }))
            }
            CellType::DELETE_GROUP => {
                let column_count = u16::from_be_bytes(buf[2..4].try_into().unwrap()) as usize;

                let mut buf = vec![0u8; 8 + column_count as usize * 2];
                reader.read_exact(&mut buf)?;

                let timestamp = u64::from_be_bytes(buf[0..8].try_into().unwrap());

                let mut columns: Vec<ColumnId> = Vec::with_capacity(column_count);
                for i in 0..column_count {
                    let column_id =
                        u16::from_be_bytes(buf[(2 * i + 10)..(2 * i + 12)].try_into().unwrap());
                    columns.push(ColumnId::from(column_id));
                }

                Ok(PartitionCell(ColumnMutation::GroupDelete {
                    columns: columns.into_boxed_slice(),
                    timestamp: Timestamp::from(timestamp),
                }))
            }
            _ => Err(anyhow!("Invalid cell type")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CellType(u16);

impl CellType {
    const PUT: CellType = CellType(0x01);
    const DELETE: CellType = CellType(0x02);
    const DELETE_GROUP: CellType = CellType(0x04);

    fn for_column_mutation(mutation: &ColumnMutation) -> Self {
        match mutation {
            ColumnMutation::Put { .. } => CellType::PUT,
            ColumnMutation::Delete { .. } => CellType::DELETE,
            ColumnMutation::GroupDelete { .. } => CellType::DELETE_GROUP,
        }
    }
}

impl Into<u16> for CellType {
    fn into(self) -> u16 {
        self.0
    }
}

impl From<u16> for CellType {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
