use thiserror::Error;

pub enum ChecksumType {
    CRC32,
}

impl TryFrom<u8> for ChecksumType {
    type Error = ChecksumTypeParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::CRC32),
            _ => Err(ChecksumTypeParseError(value)),
        }
    }
}

impl Into<u8> for ChecksumType {
    fn into(self) -> u8 {
        match self {
            Self::CRC32 => 1,
        }
    }
}

#[derive(Error, Debug)]
#[error("invalid serialized checksum type {0}")]
pub struct ChecksumTypeParseError(u8);
