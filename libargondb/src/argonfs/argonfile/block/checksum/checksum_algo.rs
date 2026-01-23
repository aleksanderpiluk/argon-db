use std::io::Write;

use super::ChecksumError;
use super::ChecksumType;

pub trait ChecksumAlgo {
    fn checksum_type(&self) -> ChecksumType;

    fn calc_checksum<W: Write>(&self, data: &[u8], out: &mut W) -> Result<usize, ChecksumError>;

    fn verify_checksum(&self, data: &[u8], checksum: &[u8]) -> Result<(), ChecksumError>;
}
