use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    hint,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    task::Waker,
};

<<<<<<< HEAD:libargondb/src/block_cache/page_buffer.rs
use crate::block_cache::{
    block_cache::{BlockCacheConfig, BlockCacheTag},
=======
use crate::argonfs::block_cache::block_view::BlockView;

use super::{
    block_cache::BlockCacheConfig,
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/page_buffer.rs
    block_lock::{BlockLock, TryExclusiveLockError},
    page::{PageHeader, PageState},
};

pub struct PageBuffer {
    page_size: usize,
    pages_total: usize,
    headers: NonNull<PageHeader>,
    blocks: NonNull<u8>,
}

impl PageBuffer {
    pub fn new(config: &BlockCacheConfig) -> Self {
        let alignment = size_of::<libc::max_align_t>();
        if config.page_size % alignment != 0 {
            panic!(
                "block size not aligned - block_size: {}, max_align_t: {}",
                config.page_size, alignment
            );
        }

        let blocks_total = config.pages_total;
        let block_size = config.page_size;

        let (headers_layout, blocks_layout) = Self::get_layouts(blocks_total, block_size);

        let (headers, blocks) = unsafe {
            let headers = NonNull::new(alloc(headers_layout) as *mut PageHeader)
                .unwrap_or_else(|| handle_alloc_error(headers_layout));
            let blocks = NonNull::new(alloc(blocks_layout))
                .unwrap_or_else(|| handle_alloc_error(blocks_layout));

            (headers, blocks)
        };

        for i in 0..blocks_total {
            let header = unsafe { headers.add(i) }.as_ptr();
            unsafe {
                *header = PageHeader {
                    lock: BlockLock::new(),
                    data: blocks.add(i * block_size),
<<<<<<< HEAD:libargondb/src/block_cache/page_buffer.rs
                    state: PageState::Free {
=======
                    buf_len: block_size,
                    state: PageState::FreelistItem {
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/page_buffer.rs
                        next_free: if i + 1 < blocks_total {
                            Some(headers.add(i + 1))
                        } else {
                            None
                        },
                    },
                }
            }
        }

        Self {
            pages_total: blocks_total,
            page_size: block_size,
            headers,
            blocks,
        }
    }

    pub fn pages_total_count(&self) -> usize {
        self.pages_total
    }

    pub fn get_header(&self, idx: usize) -> Option<NonNull<PageHeader>> {
        if idx < self.pages_total {
            Some(unsafe { self.headers.add(idx) })
        } else {
            None
        }
    }

    fn get_layouts(pages_total: usize, page_size: usize) -> (Layout, Layout) {
        let headers_layout = Layout::array::<PageHeader>(pages_total).unwrap();
        let blocks_layout = Layout::array::<u8>(pages_total * page_size).unwrap();

        (headers_layout, blocks_layout)
    }
}

impl Drop for PageBuffer {
    fn drop(&mut self) {
        let (headers_layout, blocks_layout) = Self::get_layouts(self.pages_total, self.page_size);

        unsafe {
            dealloc(self.headers.as_ptr() as *mut u8, headers_layout);
            dealloc(self.blocks.as_ptr(), blocks_layout);
        }
    }
}

<<<<<<< HEAD:libargondb/src/block_cache/page_buffer.rs
pub struct PageHeader {
    lock: BlockLock,
    data: NonNull<u8>,
    state: PageState,
}

impl PageHeader {
    pub fn is_ready(&self) -> bool {
        if let PageState::Block { .. } = self.state {
            true
        } else {
            false
        }
    }

    pub fn is_free(&self) -> bool {
        if let PageState::Free { .. } = self.state {
            true
        } else {
            false
        }
    }

    pub fn is_acquired(&self) -> bool {
        if let PageState::Acquired { .. } = self.state {
            true
        } else {
            false
        }
    }

    pub fn free(&mut self) {
        todo!()
    }

    pub fn acquire(&mut self, tag: BlockCacheTag) {
        todo!()
    }

    pub fn clear_next_free(&mut self) -> FreelistNext {
        let next_free = self.next_free;
        self.next_free = None;

        next_free
    }

    pub fn set_next_free(&mut self, next_free: FreelistNext) {
        assert_eq!(self.next_free, None);
        self.next_free = next_free;
    }

    pub fn usage_count_take(&mut self) -> u8 {
        let old_state = self.lock.load_state();
        let mut new_state = old_state;

        let usage_count = new_state.usage_count_take();

        self.lock
            .try_compare_exchange_state(old_state, new_state)
            .expect("failed while write access held");

        usage_count
    }

    pub fn usage_count(&self) -> u8 {
        self.lock.load_state().usage_count()
    }
}

#[derive(Clone)]
enum PageState {
    Acquired {
        read_dispatched: bool,
        wakers: Vec<Waker>,
    },
    Free {
        next_free: FreelistNext,
    },
    Block {
        tag: Option<BlockCacheTag>,
        block_size: usize,
        overflow_page: Option<NonNull<PageHeader>>,
    },
    OverflowPage {
        overflow_page: Option<NonNull<PageHeader>>,
    },
}

=======
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/page_buffer.rs
/**
 * Guards write access to the block(both header and data). When dropped, drops exclusive lock obtained on block.
 */
pub struct BlockExclusiveGuard(NonNull<PageHeader>);

impl BlockExclusiveGuard {
    pub fn header(&self) -> NonNull<PageHeader> {
        self.0
    }

    pub unsafe fn try_acquire_for(ptr: NonNull<PageHeader>) -> Result<Self, TryExclusiveLockError> {
        match unsafe { ptr.as_ref() }.lock.try_exclusive_lock() {
            Ok(_) => Ok(Self(ptr)),
            Err(err) => Err(err),
        }
    }

    /**
     * Acquires exclusive lock for block header. This function only guaranties that lock is obtained and it cannot
     * guarantee tag assigned to block hasn't change.
     */
    pub unsafe fn acquire_for(ptr: NonNull<PageHeader>) -> Self {
        // FIXME: This naive spin-lock implementation should be rewritten
        loop {
            match unsafe { ptr.as_ref() }.lock.try_exclusive_lock() {
                Ok(_) => return Self(ptr),
                Err(_) => {
                    hint::spin_loop();
                }
            }
        }
    }

    pub fn to_shared(self) -> BlockSharedGuard {
        todo!()
    }
}

impl Deref for BlockExclusiveGuard {
    type Target = PageHeader;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl DerefMut for BlockExclusiveGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl Drop for BlockExclusiveGuard {
    fn drop(&mut self) {
        let header = unsafe { self.0.as_ref() };

        header.lock.drop_exclusive_lock();
    }
}

unsafe impl Send for BlockExclusiveGuard {}
unsafe impl Sync for BlockExclusiveGuard {}

/**
 * Guards read access to the block(both header and data). When dropped, drops shared lock obtained on block.
 */
pub struct BlockSharedGuard(NonNull<PageHeader>);
<<<<<<< HEAD:libargondb/src/block_cache/page_buffer.rs

unsafe impl Sync for BlockSharedGuard {}
=======
>>>>>>> ae412a2 (commit):libargondb/src/argonfs/block_cache/page_buffer.rs

impl BlockSharedGuard {
    pub fn header(&self) -> NonNull<PageHeader> {
        self.0
    }

    /**
     * Acquires shared lock for block header. This function only guaranties that lock is obtained and it cannot
     * guarantee tag assigned to block hasn't change.
     */
    pub unsafe fn acquire_for(ptr: NonNull<PageHeader>, bump_usage_count: bool) -> Self {
        // FIXME: This naive spin-lock implementation should be rewritten
        loop {
            match unsafe { ptr.as_ref() }
                .lock
                .try_shared_lock(bump_usage_count)
            {
                Ok(_) => return Self(ptr),
                Err(_) => {
                    hint::spin_loop();
                }
            }
        }
    }

    pub fn to_exclusive(self) -> BlockExclusiveGuard {
        todo!()
    }

    pub fn try_to_exclusive(self) -> Result<BlockExclusiveGuard, ()> {
        todo!()
    }

    pub fn to_boxed_view(self) -> Box<BlockView> {
        todo!()
    }
}

impl Deref for BlockSharedGuard {
    type Target = PageHeader;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl Drop for BlockSharedGuard {
    fn drop(&mut self) {
        let header = unsafe { self.0.as_ref() };

        header.lock.drop_shared_lock();
    }
}

unsafe impl Send for BlockSharedGuard {}
unsafe impl Sync for BlockSharedGuard {}
