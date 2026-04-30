use std::io::Write;
use zstd::bulk::{compress, decompress};

use crate::argonfs::argonfile::block::compression::CompressionType;

use super::super::{CompressionAlgo, CompressionError};

pub struct CompressionAlgoZstd;

impl CompressionAlgo for CompressionAlgoZstd {
    fn compression_type(&self) -> CompressionType {
        CompressionType::Zstd
    }

    fn compress<W: Write>(&self, data: &[u8], out: &mut W) -> Result<(), CompressionError> {
        let tmp_buffer = compress(data, 0).map_err(CompressionError::WriteError)?;
        out.write_all(&tmp_buffer)
            .map_err(CompressionError::WriteError)
    }

    fn decompress<W: Write>(
        &self,
        data: &[u8],
        out: &mut W,
        decompressed_buffer_size: usize,
    ) -> Result<(), CompressionError> {
        let tmp_buffer =
            decompress(data, decompressed_buffer_size).map_err(CompressionError::WriteError)?;
        out.write_all(&tmp_buffer)
            .map_err(CompressionError::WriteError)
    }
}
