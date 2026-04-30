use crate::argonfile::block::compression::CompressionStrategy;

use super::CompressionType;

pub struct CompressionAlgoResolver;

impl CompressionAlgoResolver {
    pub fn for_compression_type(compression_type: CompressionType) -> CompressionStrategy {
        match compression_type {
            CompressionType::Uncompressed => CompressionStrategy::Uncompressed,
            CompressionType::Zstd => CompressionStrategy::Zstd,
        }
    }

    #[cfg(feature = "argondb_compression_zstd")]
    pub fn for_default_compression_type() -> CompressionStrategy {
        Self::for_compression_type(CompressionType::Zstd)
    }

    #[cfg(not(any(feature = "argondb_compression_zstd")))]
    pub fn for_default_compression_type() -> CompressionStrategy {
        Self::for_compression_type(CompressionType::Uncompressed)
    }
}
