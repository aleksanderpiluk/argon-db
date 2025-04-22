use std::io::Write;

use crate::{
    pointer::Pointer,
    shared::{PositionedWriter, Writer},
};

use super::{partition_row::PartitionRowWriter, Partition};

pub struct PartitionWriter;

impl Writer<Partition> for PartitionWriter {
    fn try_write<W: std::io::Write>(
        writer: &mut PositionedWriter<W>,
        partition: &Partition,
    ) -> anyhow::Result<Pointer> {
        let offset = writer.get_position();

        writer.write(&u16::to_be_bytes(partition.key.len()))?;
        writer.write(&u16::to_be_bytes(partition.rows.len() as u16))?;

        writer.write(partition.key.as_ref())?;

        for row in &partition.rows {
            PartitionRowWriter::try_write(writer, row)?;
        }

        let size = writer.get_position() - offset;

        Ok(Pointer::new(offset, size))
    }
}
