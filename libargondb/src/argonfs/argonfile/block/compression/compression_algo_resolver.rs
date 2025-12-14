use crate::argonfs::argonfile::compression::CompressionAlgo;

use super::CompressionType;
use super::algo;

pub struct CompressionAlgoResolver;

impl CompressionAlgoResolver {
    pub fn for_compression_type(compression_type: CompressionType) -> impl CompressionAlgo {
        match compression_type {
            CompressionType::Uncompressed => algo::CompressionAlgoUncompressed,
        }
    }
}
