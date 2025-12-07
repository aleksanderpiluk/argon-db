use thiserror::Error;

pub enum CompressionType {
    Uncompressed,
}

impl Into<u8> for CompressionType {
    fn into(self) -> u8 {
        match self {
            Self::Uncompressed => 1,
        }
    }
}

impl TryFrom<u8> for CompressionType {
    type Error = CompressionTypeParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Uncompressed),
            _ => Err(CompressionTypeParseError(value)),
        }
    }
}

#[derive(Error, Debug)]
#[error("invalid serialized compression type {0}")]
pub struct CompressionTypeParseError(u8);
