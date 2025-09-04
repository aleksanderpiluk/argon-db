use std::sync::LazyLock;

use crate::{limits, subsystem::io::block_cache::block_cache::BlockCache};

mod block_cache;
mod pages;

pub use pages::PageRef;

pub static BLOCK_CACHE: LazyLock<
    BlockCache<{ limits::BLOCK_CACHE_PAGE_SIZE }, { limits::BLOCK_CACHE_NUM_PAGES }>,
> = LazyLock::new(|| BlockCache::new());
