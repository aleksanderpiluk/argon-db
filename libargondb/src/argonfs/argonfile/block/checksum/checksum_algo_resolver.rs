use super::algo;
use super::{ChecksumAlgo, ChecksumType};

pub struct ChecksumAlgoResolver;

impl ChecksumAlgoResolver {
    pub fn for_checksum_type(checksum_type: ChecksumType) -> impl ChecksumAlgo {
        match checksum_type {
            ChecksumType::CRC32 => algo::ChecksumAlgoCRC32,
        }
    }
}
