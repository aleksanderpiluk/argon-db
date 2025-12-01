use std::{
    ptr::NonNull,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use super::{
    block_lock::TryExclusiveLockError,
    block_map::BlockMap,
    page::PageHeader,
    page_buffer::{BlockExclusiveGuard, PageBuffer},
};

pub type FreelistNext = Option<NonNull<PageHeader>>;

pub struct Freelist {
    next_free: Mutex<FreelistNext>,
    clock_sweep_next_victim: AtomicUsize,
}

impl Freelist {
    pub fn new(block_buffer: &PageBuffer) -> Self {
        Self {
            next_free: Mutex::new(block_buffer.get_header(0)),
            clock_sweep_next_victim: AtomicUsize::new(0),
        }
    }

    /**
     * Pops block from freelist if any available or runs clock-sweep to free mapped block
     */
    pub fn get_free_page(&self, buffer: &PageBuffer, map: &BlockMap) -> BlockExclusiveGuard {
        if let Some(block) = self.pop() {
            assert!(block.is_free());
            return block;
        }

        self.clock_sweep(buffer, map)
    }

    pub fn push_free(&self, mut block: BlockExclusiveGuard) {
        assert!(block.is_free());

        let mut next_free = self.next_free.lock().unwrap();

        block.set_state_freelist_item(*next_free);
        assert!(block.is_freelist_item());

        *next_free = Some(block.header());

        // First drop block exclusive lock, then unlock freelist
        drop(block);
        drop(next_free);
    }

    /**
     * Pops block from freelist if any available
     */
    fn pop(&self) -> Option<BlockExclusiveGuard> {
        let mut next_free = self.next_free.lock().unwrap();

        if let Some(header) = *next_free {
            // Having exclusive access to next_free pointer block must remain in free state
            let mut block = unsafe { BlockExclusiveGuard::acquire_for(header) };
            assert!(block.is_freelist_item());

            *next_free = block.set_state_free_from_freelist_item();

            assert!(block.is_free());

            Some(block)
        } else {
            None
        }
    }

    fn clock_sweep(&self, buffer: &PageBuffer, map: &BlockMap) -> BlockExclusiveGuard {
        loop {
            let Some(block) = self.clock_sweep_tick(buffer) else {
                continue;
            };

            assert!(block.is_loaded_block());
            assert_eq!(block.usage_count(), 0);

            match map.try_free_loaded_block(block) {
                Ok((block, next_overflow_page)) => {
                    self.free_overflow_pages(next_overflow_page);
                    return block;
                }
                Err(block) => {
                    // This would deadlock, so drop lock on this page and try to take another one
                    drop(block);
                    continue;
                }
            };
        }
    }

    /** Performs single "tick" of clock-sweep algorithm, if guard is returned, page should be in "LoadedBlock" state and with usage_count set to 0. */
    fn clock_sweep_tick(&self, buffer: &PageBuffer) -> Option<BlockExclusiveGuard> {
        let mut victim_idx = self.clock_sweep_next_victim.fetch_add(1, Ordering::Relaxed);
        if victim_idx >= buffer.pages_total_count() {
            let bounded_idx = victim_idx % buffer.pages_total_count();

            // TODO: Revise this orderings
            // Generally this operation is safe to fail, in case of overflow clock-sweep will behave weirdly but it shouldn't cause errors in state of system
            self.clock_sweep_next_victim.compare_exchange(
                victim_idx + 1,
                bounded_idx + 1,
                Ordering::Release,
                Ordering::Relaxed,
            );

            victim_idx = bounded_idx;
        }

        let victim_header = buffer.get_header(victim_idx).unwrap();

        let mut guard: Option<BlockExclusiveGuard> = None;
        loop {
            match unsafe { BlockExclusiveGuard::try_acquire_for(victim_header) } {
                Ok(block) => {
                    guard = Some(block);
                    break;
                }
                Err(err) => match err {
                    TryExclusiveLockError::StateChanged => {
                        continue;
                    }
                    _ => {
                        break;
                    }
                },
            }
        }

        if let Some(mut block) = guard {
            if block.is_loaded_block() {
                let usage_count = block.usage_count_take();

                if usage_count == 0 { Some(block) } else { None }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn free_overflow_pages(&self, next_overflow_page: Option<NonNull<PageHeader>>) {
        let mut next_overflow_page = next_overflow_page;
        while let Some(header) = next_overflow_page {
            let mut page = unsafe { BlockExclusiveGuard::acquire_for(header) };
            assert!(page.is_overflow_page());

            next_overflow_page = page.set_state_free_from_overflow_page();

            self.push_free(page);
        }
    }
}
