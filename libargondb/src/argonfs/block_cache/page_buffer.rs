use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    hint,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    task::Waker,
};

use crate::argonfs::block_cache::block_view::BlockView;

use super::{
    block_cache::BlockCacheConfig,
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
                    buf_len: block_size,
                    state: PageState::FreelistItem {
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
        let header = self.header();
        drop(self);

        unsafe { BlockExclusiveGuard::acquire_for(header) }
    }

    pub fn try_to_exclusive(self) -> Result<BlockExclusiveGuard, TryExclusiveLockError> {
        let header = self.header();
        drop(self);

        unsafe { BlockExclusiveGuard::try_acquire_for(header) }
    }

    pub fn to_block_view(self) -> Box<BlockView> {
        Box::new(BlockView::new(self))
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
