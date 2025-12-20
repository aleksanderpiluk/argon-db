use std::cmp::Ordering;

use crate::{
    argonfs::argonfile::{
        error::ArgonfileWriterError,
        utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
    },
    kv::{
        KVPrimaryKeyMarker, KVRangeScan,
        primary_key::{KVPrimaryKeySchema, PrimaryKeyMarkerComparator},
    },
};

pub struct Stats {
    pub bloom_filter: Box<[u8]>,
    pub min_row_key: Box<[u8]>,
    pub max_row_key: Box<[u8]>,
}

impl Stats {
    pub const MIN_SIZE_SERIALIZED: usize = 12;

    pub fn is_range_scan_intersecting(
        &self,
        schema: &KVPrimaryKeySchema,
        range_scan: &KVRangeScan,
    ) -> bool {
        let from = range_scan.from();
        let to = range_scan.to();

        // Sanity check
        assert!(PrimaryKeyMarkerComparator::cmp(schema, from, to).unwrap() != Ordering::Greater);

        let not_intersecting = (PrimaryKeyMarkerComparator::cmp(
            schema,
            to,
            &KVPrimaryKeyMarker::Key(self.min_row_key.clone()),
        )
        .unwrap()
            == Ordering::Less)
            || (PrimaryKeyMarkerComparator::cmp(
                schema,
                from,
                &KVPrimaryKeyMarker::Key(self.max_row_key.clone()),
            )
            .unwrap()
                == Ordering::Greater);

        !not_intersecting
    }

    pub fn serialize(
        writer: &mut impl ArgonfileWrite,
        stats: &Stats,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(writer);

        let min_row_key_size = stats.min_row_key.len();
        let max_row_key_size = stats.max_row_key.len();
        let bloom_filter_size = stats.bloom_filter.len();
        assert!(min_row_key_size < u16::MAX as usize);
        assert!(max_row_key_size < u16::MAX as usize);
        assert!(bloom_filter_size < u64::MAX as usize);
        writer.write(&u16::to_le_bytes(min_row_key_size as u16))?;
        writer.write(&u16::to_le_bytes(max_row_key_size as u16))?;
        writer.write(&u64::to_le_bytes(bloom_filter_size as u64))?;

        writer.write(&stats.min_row_key)?;
        writer.write(&stats.max_row_key)?;
        writer.write(&stats.bloom_filter)?;

        Ok(writer.size())
    }
}
