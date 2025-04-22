use crate::pointer::Pointer;

use super::IndexEntry;

/// An `IndexBuilder` helps to create indices to partitions in argonfile.
#[derive(Debug)]
pub struct IndexBuilder {
    indices: Vec<IndexEntry>,
    in_block: Vec<EntryInBlock>,
}

impl IndexBuilder {
    pub fn new() -> Self {
        Self {
            indices: vec![],
            in_block: vec![],
        }
    }

    /// Adds entry to vector of in-block indices with given partition offset.
    /// To transform in-block indices into proper index entries use
    /// function `commit_block`.
    pub fn add_entry_in_block(&mut self, key: Box<[u8]>, partition_offset: u32) {
        self.in_block.push(EntryInBlock {
            partition_offset,
            key,
        });
    }

    /// Transforms stored vector of in-block indices into index entries with
    /// pointer to block and in-block partition offset.
    pub fn commit_block(&mut self, block_ptr: Pointer) {
        let in_block = std::mem::replace(&mut self.in_block, vec![]);

        self.indices.extend(
            in_block
                .into_iter()
                .map(|entry| entry.into_index_entry(block_ptr)),
        );

        self.in_block.clear();
    }

    /// Closes builder, optionally performing `commit_block` for staged
    /// in-block indices. Returns vector of indices.
    ///
    /// Note: Staged in-block indices, will not be returned if not commited.
    pub fn close(mut self, block_ptr: Option<Pointer>) -> Vec<IndexEntry> {
        if let Some(block_ptr) = block_ptr {
            self.commit_block(block_ptr);
        }

        self.indices
    }
}

#[derive(Debug)]
struct EntryInBlock {
    partition_offset: u32,
    key: Box<[u8]>,
}

impl EntryInBlock {
    fn into_index_entry(&self, block_ptr: Pointer) -> IndexEntry {
        IndexEntry {
            block_ptr,
            partition_offset: self.partition_offset,
            key: self.key.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{index::IndexEntry, pointer::Pointer};

    use super::IndexBuilder;

    #[test]
    fn test_nothing_returned_if_no_commit_block() {
        // given
        let mut builder = IndexBuilder::new();

        // when
        builder.add_entry_in_block(b"some_key".as_slice().into(), 0);
        let result = builder.close(None);

        // then
        assert!(result.is_empty())
    }

    #[test]
    fn test_commit_block_works() {
        // given
        let mut builder = IndexBuilder::new();
        let key1: Box<[u8]> = b"some_key".as_slice().into();
        let key2: Box<[u8]> = b"some_key_2".as_slice().into();

        // when
        builder.add_entry_in_block(key1.clone(), 0);
        builder.add_entry_in_block(key2.clone(), 10);
        builder.commit_block(Pointer::new(0, 30));
        let result = builder.close(None);

        // then
        assert_eq!(
            result,
            vec![
                IndexEntry {
                    block_ptr: Pointer::new(0, 30),
                    key: key1,
                    partition_offset: 0
                },
                IndexEntry {
                    block_ptr: Pointer::new(0, 30),
                    key: key2,
                    partition_offset: 10
                }
            ]
        );
    }
}
