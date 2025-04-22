use crate::shared::Writer;

use super::IndexEntry;

pub struct IndexEntryWriter {}

impl Writer<IndexEntry> for IndexEntryWriter {
    fn try_write<W: std::io::Write>(
        writer: &mut crate::shared::PositionedWriter<W>,
        pointer: &IndexEntry,
    ) -> anyhow::Result<crate::pointer::Pointer> {
        todo!()
    }
}
