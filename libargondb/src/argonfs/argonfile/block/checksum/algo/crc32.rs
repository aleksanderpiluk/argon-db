use std::io::Write;

use crc32c::crc32c;

use crate::argonfs::argonfile::checksum::{ChecksumAlgo, ChecksumError, ChecksumType};

pub struct ChecksumAlgoCRC32;

impl ChecksumAlgo for ChecksumAlgoCRC32 {
    fn checksum_type(&self) -> ChecksumType {
        ChecksumType::CRC32
    }

    fn calc_checksum<W: Write>(&self, data: &[u8], out: &mut W) -> Result<usize, ChecksumError> {
        let checksum = crc32c(data);
        let checksum_bytes = u32::to_le_bytes(checksum);

        out.write_all(&checksum_bytes)
            .map_err(|e| ChecksumError::WriteError(e))?;

        Ok(checksum_bytes.len())
    }

    fn verify_checksum(&self, data: &[u8], checksum: &[u8]) -> Result<(), ChecksumError> {
        let checksum_bytes = checksum
            .try_into()
            .map_err(|_| ChecksumError::ChecksumMalformed)?;

        let given_checksum = u32::from_be_bytes(checksum_bytes);
        let data_checksum = crc32c(data);

        if data_checksum == given_checksum {
            Ok(())
        } else {
            Err(ChecksumError::ValidationFailed)
        }
    }
}
