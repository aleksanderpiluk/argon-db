use std::io::Write;

use crate::{
    argonfile::block::compression::algo::{CompressionAlgoUncompressed, CompressionAlgoZstd},
    argonfs::argonfile::block::compression::CompressionType,
};

use super::CompressionError;

pub trait CompressionAlgo {
    fn compression_type(&self) -> CompressionType;

    fn compress<W: Write>(&self, data: &[u8], out: &mut W) -> Result<(), CompressionError>;

    fn decompress<W: Write>(
        &self,
        data: &[u8],
        out: &mut W,
        decompressed_buffer_size: usize,
    ) -> Result<(), CompressionError>;
}

pub enum CompressionStrategy {
    Uncompressed,
    Zstd,
}

impl CompressionAlgo for CompressionStrategy {
    fn compression_type(&self) -> CompressionType {
        match self {
            Self::Uncompressed => CompressionType::Uncompressed,
            Self::Zstd => CompressionType::Zstd,
        }
    }

    fn compress<W: Write>(&self, data: &[u8], out: &mut W) -> Result<(), CompressionError> {
        match self {
            Self::Uncompressed => CompressionAlgoUncompressed.compress(data, out),
            Self::Zstd => CompressionAlgoZstd.compress(data, out),
        }
    }

    fn decompress<W: Write>(
        &self,
        data: &[u8],
        out: &mut W,
        decompressed_buffer_size: usize,
    ) -> Result<(), CompressionError> {
        match self {
            Self::Uncompressed => {
                CompressionAlgoUncompressed.decompress(data, out, decompressed_buffer_size)
            }
            Self::Zstd => CompressionAlgoZstd.decompress(data, out, decompressed_buffer_size),
        }
    }
}
