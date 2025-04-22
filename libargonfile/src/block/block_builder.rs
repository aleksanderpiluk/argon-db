use std::io::{Cursor, Write};

use crate::shared::PositionedWriter;

use super::{Block, BlockIdentifier};

/// A `BlockBuilder` provides functionality to construct argonfile blocks of
/// desired size or bigger by writing block data to builder buffer.
///
/// After writing to internal buffer, the code should call `next` function to
/// create new block if desired size is exceeded.
#[derive(Debug)]
pub struct BlockBuilder {
    block_identifier: BlockIdentifier,
    desired_data_size: u32,
    data_cache: PositionedWriter<Vec<u8>>,
}

impl BlockBuilder {
    /// Creates new builder with identifer of returned blocks and desired size
    /// of block data.
    pub fn new(block_identifier: BlockIdentifier, desired_data_size: u32) -> Self {
        Self {
            block_identifier,
            desired_data_size,
            //TODO: Change capacity of vector
            data_cache: PositionedWriter::new(Vec::with_capacity(1024)),
        }
    }

    /// Closes the builder. If internal buffer contains any data, it will
    /// be returned as a final block.
    pub fn close(builder: Self) -> Option<Block> {
        if builder.data_cache.get_position() > 0 {
            Some(
                Block::new(
                    builder.block_identifier,
                    builder.data_cache.into().into_boxed_slice(),
                )
                .unwrap(),
            )
        } else {
            None
        }
    }

    /// Checks if size of internal buffer exceeds desired data size. If true,
    /// then the new block is created and returned.
    pub fn next(&mut self) -> Option<Block> {
        if self.data_cache.get_position() >= self.desired_data_size as usize {
            let data_cache = std::mem::replace(
                &mut self.data_cache,
                PositionedWriter::new(Vec::with_capacity(1024)),
            );
            let block =
                Block::new(self.block_identifier, data_cache.into().into_boxed_slice()).unwrap();

            Some(block)
        } else {
            None
        }
    }
}

impl AsMut<PositionedWriter<Vec<u8>>> for BlockBuilder {
    fn as_mut(&mut self) -> &mut PositionedWriter<Vec<u8>> {
        &mut self.data_cache
    }
}

#[cfg(test)]
mod tests {
    use crate::block::BlockIdentifier;

    use super::BlockBuilder;

    #[test]
    fn test_next_on_empty_buffer() {
        // given
        let mut builder = BlockBuilder::new(BlockIdentifier::DATA_BLOCK, 10);

        // when
        let result = builder.next();

        // then
        assert!(result.is_none());
    }
}
