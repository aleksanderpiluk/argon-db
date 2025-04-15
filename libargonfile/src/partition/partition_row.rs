use std::{cell, io::Write};

use anyhow::{Context, Ok};
use libargondb::ClusteringKey;

use crate::{
    pointer::Pointer,
    shared::{Reader, Writer},
};

use super::partition_cell::{PartitionCell, PartitionCellReader, PartitionCellWriter};

#[derive(Debug, Clone)]
pub struct PartitionRow {
    clustering_key: ClusteringKey,
    cells: Box<[PartitionCell]>,
}

#[derive(Debug)]
pub struct PartitionRowMut {
    clustering_key: ClusteringKey,
    cells: Vec<PartitionCell>,
}

impl PartitionRowMut {
    pub fn with_key(clustering_key: ClusteringKey) -> Self {
        Self {
            clustering_key,
            cells: vec![],
        }
    }

    pub fn push_cell(&mut self, cell: PartitionCell) {
        self.cells.push(cell);
    }
}

impl Into<PartitionRow> for PartitionRowMut {
    fn into(self) -> PartitionRow {
        PartitionRow {
            clustering_key: self.clustering_key,
            cells: self.cells.into_boxed_slice(),
        }
    }
}

pub struct PartitionRowWriter;

impl Writer<PartitionRow> for PartitionRowWriter {
    fn try_write<W: std::io::Write>(
        writer: &mut crate::shared::PositionedWriter<W>,
        row: &PartitionRow,
    ) -> anyhow::Result<crate::pointer::Pointer> {
        let offset = writer.get_position();

        writer.write(&u16::to_be_bytes(row.clustering_key.inner().len() as u16))?;
        writer.write(&u16::to_be_bytes(row.cells.len() as u16))?;

        writer.write(row.clustering_key.inner())?;

        for cell in &row.cells {
            PartitionCellWriter::try_write(writer, cell)?;
        }

        let size: usize = writer.get_position() - offset;

        Ok(Pointer::new(offset, size))
    }
}

pub struct PartitionRowReader;

impl Reader<PartitionRow> for PartitionRowReader {
    fn try_read<R: std::io::Read>(reader: &mut R) -> anyhow::Result<PartitionRow> {
        let mut buf = vec![0u8; 4];
        reader.read_exact(&mut buf)?;

        let key_len = u16::from_be_bytes(buf[0..2].try_into().unwrap());
        let cells_count = u16::from_be_bytes(buf[2..4].try_into().unwrap());

        let mut buf = vec![0u8; key_len as usize].into_boxed_slice();
        reader.read_exact(&mut buf)?;

        let mut cells: Vec<PartitionCell> = Vec::with_capacity(cells_count as usize);
        for i in 0..cells_count {
            cells[i as usize] = PartitionCellReader::try_read(reader)?;
        }

        Ok(PartitionRow {
            clustering_key: ClusteringKey::from(buf),
            cells: cells.into_boxed_slice(),
        })
    }
}
