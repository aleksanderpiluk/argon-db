use std::io::Write;

use crate::argonfs::argonfile::block::compression::CompressionType;

use super::CompressionError;

pub trait CompressionAlgo {
    fn compression_type(&self) -> CompressionType;

    fn compress<W: Write>(&self, data: &[u8], out: &mut W) -> Result<(), CompressionError>;

    fn decompress<W: Write>(&self, data: &[u8], out: &mut W) -> Result<(), CompressionError>;
}
