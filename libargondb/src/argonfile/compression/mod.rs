use crate::argonfile::{
    compression::no_compression::ArgonfileNoCompression, config::ArgonfileConfig,
};

mod no_compression;

pub trait ArgonfileCompressionStrategy {
    fn compression_type(&self) -> u16;

    fn compress(data: &[u8]) -> Box<[u8]>;

    fn clone(&self) -> Self;
}

pub struct ArgonfileCompressionStrategyFactory;

impl ArgonfileCompressionStrategyFactory {
    pub fn from_config(config: &ArgonfileConfig) -> impl ArgonfileCompressionStrategy {
        ArgonfileNoCompression
    }
}
