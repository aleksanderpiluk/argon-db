use bytes::Buf;

use crate::argonfs::argonfile::{checksum::crc32::ArgonfileCRC32Checksum, config::ArgonfileConfig};

mod crc32;

pub trait ArgonfileChecksumStrategy {
    fn checksum_type(&self) -> u8;

    fn calc_checksum(data: &[u8]) -> Box<[u8]>;

    fn verify_checksum<B: Buf>(
        &self,
        data: &mut B,
        checksum_bytes: &[u8],
    ) -> Result<bool, ArgonfileChecksumStrategyError>;

    fn clone(&self) -> Self;
}

pub struct ArgonfileChecksumStrategyFactory;

impl ArgonfileChecksumStrategyFactory {
    pub fn from_config(config: &ArgonfileConfig) -> impl ArgonfileChecksumStrategy {
        todo!();
        ArgonfileCRC32Checksum
    }

    pub fn from_checksum_type(checksum_type: u8) -> impl ArgonfileChecksumStrategy {
        todo!();
        ArgonfileCRC32Checksum
    }
}

#[derive(Debug)]
pub enum ArgonfileChecksumStrategyError {
    ChecksumMalformed,
}
