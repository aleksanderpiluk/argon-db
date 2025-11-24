use std::io::Write;

use super::ArgonfileCompressionStrategy;

pub struct ArgonfileNoCompression;

impl ArgonfileCompressionStrategy for ArgonfileNoCompression {
    fn compression_type(&self) -> u16 {
        todo!()
    }

    fn compress(data: &[u8]) -> Box<[u8]> {
        Box::from(data)
    }

    fn decompress<W: Write>(&self, data: &[u8], out: &mut W) {
        out.write(data);
    }

    fn clone(&self) -> Self {
        Self
    }
}
