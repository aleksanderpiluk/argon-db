use crate::argonfs::argonfile::block::BlockPointer;
use crate::argonfs::argonfile::{error::ArgonfileParseResult, summary::SummaryIndex};

use super::super::parse_utils::ensure_min_size;
use super::summary_index_entry::SummaryIndexEntry;

pub struct SummaryParser;

impl SummaryParser {
    pub fn parse(buf: &[u8]) -> ArgonfileParseResult<SummaryIndex> {
        ensure_min_size(buf.len(), SummaryIndex::MIN_SIZE_SERIALIZED)?;

        let items_count = u64::from_le_bytes(buf[0..8].try_into().unwrap());

        let buf_items = &buf[8..];
        let mut entries_iter = SummaryIndexEntryParserIter { buf: buf_items };

        let mut entries = Vec::with_capacity(items_count as _);
        for _ in 0..items_count {
            let entry = entries_iter.next()?;

            entries.push(entry);
        }

        Ok(SummaryIndex { entries })
    }
}

struct SummaryIndexEntryParserIter<'a> {
    buf: &'a [u8],
}

impl SummaryIndexEntryParserIter<'_> {
    fn next(&mut self) -> ArgonfileParseResult<SummaryIndexEntry> {
        let buf = self.buf;
        ensure_min_size(buf.len(), SummaryIndexEntry::MIN_SIZE_SERIALIZED)?;

        let block_ptr = BlockPointer::parse(&buf[0..BlockPointer::SERIALIZED_SIZE])?;

        let buf = &buf[BlockPointer::SERIALIZED_SIZE..];
        let key_size = u16::from_le_bytes(buf[0..2].try_into().unwrap()) as usize;

        let buf = &buf[2..];
        ensure_min_size(buf.len(), key_size)?;
        let key = Box::<[u8]>::from(&buf[0..key_size]);

        self.buf = &buf[key_size..];
        Ok(SummaryIndexEntry { block_ptr, key })
    }
}
