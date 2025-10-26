mod crc32;

pub trait ArgonfileChecksumStrategy {
    fn checksum_type() -> u8;

    fn calc_checksum(data: &[u8]) -> Box<[u8]>;

    fn verify_checksum(
        data: &[u8],
        checksum_bytes: &[u8],
    ) -> Result<bool, ArgonfileChecksumStrategyError>;
}

pub enum ArgonfileChecksumStrategyError {
    ChecksumMalformed,
}
