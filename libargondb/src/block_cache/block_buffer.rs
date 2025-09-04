use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::block_cache::{block_lock::BlockLock, freelist::FreelistNext};

pub struct BlockBuffer {
    blocks_total_count: usize,
    headers: NonNull<BlockHeader>,
}

impl BlockBuffer {
    pub fn new() -> Self {
        todo!()
    }

    pub fn blocks_total_count(&self) -> usize {
        self.blocks_total_count
    }

    pub fn get_header(&self, idx: usize) -> Option<NonNull<BlockHeader>> {
        if idx < self.blocks_total_count {
            Some(unsafe { self.headers.add(idx) })
        } else {
            None
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

    /**
     * Acquires exclusive lock for block header. This function only guaranties that lock is obtained and it cannot
     * guarantee tag assigned to block hasn't change.
     */
    pub unsafe fn acquire_for(ptr: NonNull<BlockHeader>) -> Self {
        todo!()
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
    pub unsafe fn acquire_for(ptr: NonNull<BlockHeader>) -> Self {
        todo!()
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
