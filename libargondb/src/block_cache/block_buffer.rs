use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    hint,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::block_cache::{
    block_cache::BlockCacheConfig,
    block_lock::{BlockLock, TryExclusiveLockError},
    freelist::FreelistNext,
};

pub struct BlockBuffer {
    block_size: usize,
    blocks_total: usize,
    headers: NonNull<BlockHeader>,
    blocks: NonNull<u8>,
}

impl BlockBuffer {
    pub fn new(config: &BlockCacheConfig) -> Self {
        let alignment = size_of::<libc::max_align_t>();
        if config.block_size % alignment != 0 {
            panic!(
                "block size not aligned - block_size: {}, max_align_t: {}",
                config.block_size, alignment
            );
        }

        let blocks_total = config.blocks_total;
        let block_size = config.block_size;

        let (headers_layout, blocks_layout) = Self::get_layouts(blocks_total, block_size);

        let (headers, blocks) = unsafe {
            let headers = NonNull::new(alloc(headers_layout) as *mut BlockHeader)
                .unwrap_or_else(|| handle_alloc_error(headers_layout));
            let blocks = NonNull::new(alloc(blocks_layout))
                .unwrap_or_else(|| handle_alloc_error(blocks_layout));

            (headers, blocks)
        };

        for i in 0..blocks_total {
            let header = unsafe { headers.add(i) }.as_ptr();
            unsafe {
                *header = BlockHeader {
                    data: blocks.add(i * block_size),
                    state: BlockState::Free,
                    tag: None,
                    lock: BlockLock::new(),
                    next_free: if i + 1 < blocks_total {
                        Some(headers.add(i + 1))
                    } else {
                        None
                    },
                }
            }
        }

        Self {
            blocks_total,
            block_size,
            headers,
            blocks,
        }
    }

    pub fn blocks_total_count(&self) -> usize {
        self.blocks_total
    }

    pub fn get_header(&self, idx: usize) -> Option<NonNull<BlockHeader>> {
        if idx < self.blocks_total {
            Some(unsafe { self.headers.add(idx) })
        } else {
            None
        }
    }

    fn get_layouts(blocks_total: usize, block_size: usize) -> (Layout, Layout) {
        let headers_layout = Layout::array::<BlockHeader>(blocks_total).unwrap();
        let blocks_layout = Layout::array::<u8>(blocks_total * block_size).unwrap();

        (headers_layout, blocks_layout)
    }
}

impl Drop for BlockBuffer {
    fn drop(&mut self) {
        let (headers_layout, blocks_layout) = Self::get_layouts(self.blocks_total, self.block_size);

        unsafe {
            dealloc(self.headers.as_ptr() as *mut u8, headers_layout);
            dealloc(self.blocks.as_ptr(), blocks_layout);
        }
    }
}

pub struct BlockHeader {
    lock: BlockLock,
    data: NonNull<u8>,
    state: BlockState,
    tag: Option<u64>,
    next_free: FreelistNext,
}

impl BlockHeader {
    pub fn is_free(&self) -> bool {
        self.state == BlockState::Free
    }

    pub fn is_acquired(&self) -> bool {
        self.state == BlockState::Acquired
    }

    pub fn is_loaded(&self) -> bool {
        self.state == BlockState::Loaded
    }

    pub fn next_free(&self) -> FreelistNext {
        self.next_free
    }

    pub fn tag(&self) -> Option<u64> {
        self.tag
    }

    pub fn free(&mut self) {
        todo!()
    }

    pub fn acquire(&mut self, tag: u64) {
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockState {
    Free,
    Acquired,
    Loaded,
}

/**
 * Guards write access to the block(both header and data). When dropped, drops exclusive lock obtained on block.
 */
pub struct BlockExclusiveGuard(NonNull<BlockHeader>);

impl BlockExclusiveGuard {
    pub fn header(&self) -> NonNull<BlockHeader> {
        self.0
    }

    pub unsafe fn try_acquire_for(
        ptr: NonNull<BlockHeader>,
    ) -> Result<Self, TryExclusiveLockError> {
        match unsafe { ptr.as_ref() }.lock.try_exclusive_lock() {
            Ok(_) => Ok(Self(ptr)),
            Err(err) => Err(err),
        }
    }

    /**
     * Acquires exclusive lock for block header. This function only guaranties that lock is obtained and it cannot
     * guarantee tag assigned to block hasn't change.
     */
    pub unsafe fn acquire_for(ptr: NonNull<BlockHeader>) -> Self {
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
}

impl Deref for BlockExclusiveGuard {
    type Target = BlockHeader;

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

/**
 * Guards read access to the block(both header and data). When dropped, drops shared lock obtained on block.
 */
pub struct BlockSharedGuard(NonNull<BlockHeader>);

impl BlockSharedGuard {
    pub fn header(&self) -> NonNull<BlockHeader> {
        self.0
    }

    /**
     * Acquires shared lock for block header. This function only guaranties that lock is obtained and it cannot
     * guarantee tag assigned to block hasn't change.
     */
    pub unsafe fn acquire_for(ptr: NonNull<BlockHeader>, bump_usage_count: bool) -> Self {
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
}

impl Deref for BlockSharedGuard {
    type Target = BlockHeader;

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
