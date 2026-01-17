use bloomfilter::Bloom;

use super::super::parse_utils::ensure_min_size;
use crate::{
    argonfile::error::ArgonfileParseError,
    argonfs::argonfile::{error::ArgonfileParseResult, stats::Stats},
};

pub struct StatsParser;

impl StatsParser {
    pub fn parse(buf: &[u8]) -> ArgonfileParseResult<Stats> {
        ensure_min_size(buf.len(), Stats::MIN_SIZE_SERIALIZED)?;

        let min_row_key_size = u16::from_le_bytes(buf[0..2].try_into().unwrap()) as usize;
        let max_row_key_size = u16::from_le_bytes(buf[2..4].try_into().unwrap()) as usize;
        let bloom_filter_size = u64::from_le_bytes(buf[4..12].try_into().unwrap()) as usize;

        let buf = &buf[12..];
        ensure_min_size(buf.len(), min_row_key_size)?;
        let min_row_key = Box::<[u8]>::from(&buf[0..min_row_key_size]);

        let buf = &buf[min_row_key_size..];
        ensure_min_size(buf.len(), max_row_key_size)?;
        let max_row_key = Box::<[u8]>::from(&buf[0..max_row_key_size]);

        let buf = &buf[max_row_key_size..];
        ensure_min_size(buf.len(), bloom_filter_size)?;
        let bloom_filter = Box::<[u8]>::from(&buf[0..bloom_filter_size]);

        let bloom_filter =
            Bloom::<[u8]>::from_bytes(bloom_filter.to_vec()).map_err(|_| ArgonfileParseError)?;

        Ok(Stats {
            bloom_filter,
            min_row_key,
            max_row_key,
        })
    }
}
