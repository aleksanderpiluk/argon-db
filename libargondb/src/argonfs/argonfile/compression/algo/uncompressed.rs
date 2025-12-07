use std::io::Write;

use crate::argonfs::argonfile::compression::{CompressionAlgo, CompressionError};

pub struct CompressionAlgoUncompressed;

impl CompressionAlgo for CompressionAlgoUncompressed {
    fn compress<W: Write>(&self, data: &[u8], out: &mut W) -> Result<(), CompressionError> {
        out.write_all(data)
            .map_err(|e| CompressionError::WriteError(e))
    }

    fn decompress<W: Write>(&self, data: &[u8], out: &mut W) -> Result<(), CompressionError> {
        out.write_all(data)
            .map_err(|e| CompressionError::WriteError(e))
    }
}
