use std::io::Write;

mod no_compression;

pub use no_compression::ArgonfileNoCompression;

use crate::argonfs::argonfile::config::ArgonfileConfig;

pub trait ArgonfileCompressionStrategy {
    fn compression_type(&self) -> u16;

    fn compress(data: &[u8]) -> Box<[u8]>;

    fn decompress<W: Write>(&self, data: &[u8], out: &mut W);

    fn clone(&self) -> Self;
}

pub struct ArgonfileCompressionStrategyFactory;

impl ArgonfileCompressionStrategyFactory {
    pub fn from_config(config: &ArgonfileConfig) -> impl ArgonfileCompressionStrategy {
        ArgonfileNoCompression
    }
}
