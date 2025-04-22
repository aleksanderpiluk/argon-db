use std::io::Write;

use anyhow::Ok;

use crate::{
    pointer::{Pointer, PointerWriter},
    shared::Writer,
};

use super::SummaryEntry;

pub struct SummaryEntryWriter;

impl Writer<SummaryEntry> for SummaryEntryWriter {
    fn try_write<W: std::io::Write>(
        writer: &mut crate::shared::PositionedWriter<W>,
        entry: &SummaryEntry,
    ) -> anyhow::Result<crate::pointer::Pointer> {
        let offset = writer.get_position();

        writer.write(&u16::to_be_bytes(entry.key.len() as u16))?;
        writer.write(&entry.key)?;
        PointerWriter::try_write(writer, &entry.block_ptr)?;

        let size = writer.get_position() - offset;
        Ok(Pointer::new(offset, size))
    }
}
