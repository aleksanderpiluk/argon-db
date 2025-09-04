use std::{
    ptr::NonNull,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use crate::block_cache::{
    block_buffer::{BlockBuffer, BlockExclusiveGuard, BlockHeader},
    block_map::BlockMap,
};

pub type FreelistNext = Option<NonNull<BlockHeader>>;

pub struct Freelist {
    next_free: Mutex<FreelistNext>,
    clock_sweep_next_victim: AtomicUsize,
}

impl Freelist {
    /**
     * Pops block from freelist if any available or runs clock-sweep to free mapped block
     */
    pub fn get_free_page(&self, buffer: &BlockBuffer, map: &BlockMap) -> BlockExclusiveGuard {
        if let Some(block) = self.pop() {
            return block;
        }

        self.clock_sweep(buffer, map)
    }

    pub fn push_free(&self, mut block: BlockExclusiveGuard) {
        assert!(block.is_free());
        assert_eq!(block.next_free(), None);

        let mut next_free = self.next_free.lock().unwrap();

        block.set_next_free(*next_free);
        *next_free = Some(block.header());

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
            assert!(block.is_free());

            *next_free = block.clear_next_free();
            assert_eq!(block.next_free(), None);

            Some(block)
        } else {
            None
        }
    }

    fn clock_sweep(&self, buffer: &BlockBuffer, map: &BlockMap) -> BlockExclusiveGuard {
        loop {
            let Some(block) = self.clock_sweep_tick(buffer) else {
                continue;
            };

            assert!(block.is_loaded());
            assert_eq!(block.next_free(), None);
            assert!(block.tag().is_some());
            // TODO: asserts that it's not used and all

            let block = map.free_assigned_block(block);

            return block;
        }
    }

    fn clock_sweep_tick(&self, buffer: &BlockBuffer) -> Option<BlockExclusiveGuard> {
        let mut victim_idx = self.clock_sweep_next_victim.fetch_add(1, Ordering::Relaxed);
        if victim_idx >= buffer.blocks_total_count() {
            let bounded_idx = victim_idx % buffer.blocks_total_count();

            // TODO: Revise this orderings
            // Generally this operation is safe to fail, in case of overflow clock-sweep will behave weirdly but it shouldn't cause errors in state of system
            self.clock_sweep_next_victim.compare_exchange(
                victim_idx + 1,
                bounded_idx + 1,
                Ordering::Release,
                Ordering::Release,
            );

            victim_idx = bounded_idx;
        }

        let victim_header = buffer.get_header(victim_idx).unwrap();
        let block = unsafe { BlockExclusiveGuard::acquire_for(victim_header) }; // Any changes while waiting for lock are irrelevant

        todo!("block check and other stuff");
    }
}
