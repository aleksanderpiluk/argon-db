mod no_compression;

pub trait ArgonfileCompressionStrategy {
    fn compression_type() -> u16;

    fn compress(data: &[u8]) -> Box<[u8]>;
}
