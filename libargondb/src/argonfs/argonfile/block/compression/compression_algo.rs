use std::io::Write;

use super::CompressionError;

pub trait CompressionAlgo {
    fn compress<W: Write>(&self, data: &[u8], out: &mut W) -> Result<(), CompressionError>;

    fn decompress<W: Write>(&self, data: &[u8], out: &mut W) -> Result<(), CompressionError>;
}
