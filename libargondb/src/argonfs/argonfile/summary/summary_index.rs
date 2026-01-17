use crate::{
    argonfs::argonfile::{
        BlockPointer,
        error::ArgonfileWriterError,
        summary::SummaryIndexEntry,
        utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
    },
    kv::{
        KVPrimaryKeyMarker, KVRangeScan,
        primary_key::{KVPrimaryKeyComparator, KVPrimaryKeySchema, PrimaryKeyMarkerComparator},
    },
};

#[derive(Debug)]
pub struct SummaryIndex {
    pub entries: Vec<SummaryIndexEntry>,
}

impl SummaryIndex {
    pub const MIN_SIZE_SERIALIZED: usize = 8;

    pub fn get_blocks_for_range_scan(
        &self,
        schema: &KVPrimaryKeySchema,
        range_scan: &KVRangeScan,
    ) -> Vec<BlockPointer> {
        let from = range_scan.from();
        let to = range_scan.to();

        let from_find = self.entries.binary_search_by(|entry| {
            PrimaryKeyMarkerComparator::cmp(
                schema,
                &KVPrimaryKeyMarker::Key(entry.key.clone()),
                from,
            )
            .unwrap()
        });

        let start = match from_find {
            Ok(idx) => idx,
            Err(idx) => {
                if idx >= self.entries.len() {
                    self.entries.len() - 1
                } else if idx == 0 {
                    0
                } else {
                    idx - 1
                }
            }
        };

        let to_find = self.entries.binary_search_by(|entry| {
            PrimaryKeyMarkerComparator::cmp(schema, &KVPrimaryKeyMarker::Key(entry.key.clone()), to)
                .unwrap()
        });

        let end = match to_find {
            Ok(idx) => idx,
            Err(idx) => idx.min(self.entries.len() - 1),
        };

        self.entries[start..=end]
            .iter()
            .map(|entry| entry.block_ptr)
            .collect()
    }

    pub fn serialize(
        writer: &mut impl ArgonfileWrite,
        summary_index: &SummaryIndex,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(writer);

        let entry_count = summary_index.entries.len() as u64;
        writer.write(&u64::to_le_bytes(entry_count))?;

        for entry in &summary_index.entries {
            SummaryIndexEntry::serialize(&mut writer, entry)?;
        }

        Ok(writer.size())
    }
}
