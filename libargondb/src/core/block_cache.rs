use crate::core::buffer_pool::PageStatus;

use super::buffer_pool::{BufferPool, Page};

struct BlockCache {
    pool: BufferPool<BlockId>,
}

impl BlockCache {
    fn new(cache_size: usize) -> Result<Self, ()> {
        let pool = BufferPool::<BlockId>::new(cache_size)?;

        Ok(Self { pool })
    }

    fn get_block(&self, id: BlockId) -> Block {
        if let Some(page) = self.pool.try_get_page(id) {
            return page;
        }

        let page = match self.pool.get_page(id) {
            (PageStatus::EXISTS, page) => page,
            (PageStatus::INVALID, page) => {
                // todo!("Populate this page");
                page
            }
        };

        page
    }
}

type BlockId = u64;

type Block = Page<BlockId>;

#[cfg(test)]
mod tests {
    use super::BlockCache;

    #[test]
    fn test() {
        let cache = BlockCache::new(1024 * 1024).unwrap();

        let block_id = 10;
        let block = cache.get_block(block_id);
        let data = "ala ma kota".as_bytes();

        {
            let mut write = block.write();
            write[0..11].copy_from_slice(data);
        }

        {
            let read = block.read();
            assert_eq!(&read[0..11], data);
            println!("{:?}", String::from_utf8_lossy(&read[0..20]));
        }
    }
}
