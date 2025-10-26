use super::{ArgonfileChecksumStrategy, ArgonfileChecksumStrategyError};
use crc32c::crc32c;

pub struct ArgonfileCRC32Checksum;

impl ArgonfileChecksumStrategy for ArgonfileCRC32Checksum {
    fn checksum_type() -> u8 {
        todo!()
    }

    fn calc_checksum(data: &[u8]) -> Box<[u8]> {
        let checksum = crc32c(data);
        let checksum_bytes = u32::to_le_bytes(checksum);

        Box::from(checksum_bytes)
    }

    fn verify_checksum(
        data: &[u8],
        checksum_bytes: &[u8],
    ) -> Result<bool, ArgonfileChecksumStrategyError> {
        let checksum_bytes = checksum_bytes
            .try_into()
            .map_err(|_| ArgonfileChecksumStrategyError::ChecksumMalformed)?;

        let given_checksum = u32::from_be_bytes(checksum_bytes);
        let data_checksum = crc32c(data);

        Ok(data_checksum == given_checksum)
    }
}
