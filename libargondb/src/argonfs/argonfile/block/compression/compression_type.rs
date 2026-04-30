use thiserror::Error;

use crate::argonfs::argonfile::error::ArgonfileParseError;

#[derive(Debug, Clone, Copy)]
pub enum CompressionType {
    Uncompressed,
    Zstd,
}

impl Into<u8> for CompressionType {
    fn into(self) -> u8 {
        match self {
            Self::Uncompressed => 1,
            Self::Zstd => 2,
        }
    }
}

impl TryFrom<u8> for CompressionType {
    type Error = CompressionTypeParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Uncompressed),
            2 => Ok(Self::Zstd),
            _ => Err(CompressionTypeParseError(value)),
        }
    }
}

impl std::fmt::Display for CompressionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uncompressed => write!(f, "Uncompressed"),
            Self::Zstd => write!(f, "zstd"),
        }
    }
}

#[derive(Error, Debug)]
#[error("invalid serialized compression type {0}")]
pub struct CompressionTypeParseError(u8);

impl From<CompressionTypeParseError> for ArgonfileParseError {
    fn from(value: CompressionTypeParseError) -> Self {
        Self
    }
}
