use super::ArgonfileCompressionStrategy;

pub struct ArgonfileNoComporession;

impl ArgonfileCompressionStrategy for ArgonfileNoComporession {
    fn compression_type() -> u16 {
        todo!()
    }

    fn compress(data: &[u8]) -> Box<[u8]> {
        Box::from(data)
    }
}
