use crate::shared::Reader;

use super::IndexEntry;

pub struct IndexEntryReader;

impl Reader<IndexEntry> for IndexEntryReader {
    fn try_read<R: std::io::Read>(reader: &mut R) -> anyhow::Result<IndexEntry> {
        todo!()
    }
}
