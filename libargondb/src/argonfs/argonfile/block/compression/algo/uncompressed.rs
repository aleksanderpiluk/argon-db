use std::io::Write;

use crate::argonfs::argonfile::block::compression::CompressionType;

use super::super::{CompressionAlgo, CompressionError};

pub struct CompressionAlgoUncompressed;

impl CompressionAlgo for CompressionAlgoUncompressed {
    fn compression_type(&self) -> CompressionType {
        CompressionType::Uncompressed
    }

    fn compress<W: Write>(&self, data: &[u8], out: &mut W) -> Result<(), CompressionError> {
        out.write_all(data)
            .map_err(|e| CompressionError::WriteError(e))
    }

    fn decompress<W: Write>(
        &self,
        data: &[u8],
        out: &mut W,
        decompressed_buffer_size: usize,
    ) -> Result<(), CompressionError> {
        out.write_all(data)
            .map_err(|e| CompressionError::WriteError(e))
    }
}
