use thiserror::Error;

use crate::argonfs::argonfile::error::ArgonfileParseError;

#[derive(Debug, Clone, Copy)]
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

impl std::fmt::Display for ChecksumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CRC32 => write!(f, "CRC32"),
        }
    }
}

#[derive(Error, Debug)]
#[error("invalid serialized checksum type {0}")]
pub struct ChecksumTypeParseError(u8);

impl From<ChecksumTypeParseError> for ArgonfileParseError {
    fn from(value: ChecksumTypeParseError) -> Self {
        Self
    }
}
