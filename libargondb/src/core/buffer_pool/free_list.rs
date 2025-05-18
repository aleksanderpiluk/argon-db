use std::{cell::UnsafeCell, ptr::NonNull, sync::atomic::AtomicUsize};

use super::PageDescriptor;

struct FreeList<T> {
    next_free: UnsafeCell<Option<NonNull<PageDescriptor<T>>>>,

    next_victim_page_id: AtomicUsize,
}

impl<T> FreeList<T> {
    /// Returns unused page. If there are no pages in free list, clock-sweep page
    /// replacement is run.
    fn get_page(&self) {}

    fn free_page(&self) {}

    unsafe fn _internal_acquire_unused_page(&self) -> Option<NonNull<PageDescriptor<T>>> {
        if let Some(descriptor) = unsafe { *self.next_free.get() } {
            unsafe {
                let next_free = descriptor.as_ref().next_free;
                *self.next_free.get() = next_free;
                (*descriptor.as_ptr()).next_free = None;
            }

            return Some(descriptor);
        }

        // TODO: LRU/Clocksweep here

        None
    }

    fn _internal_clocksweep(&self) {
        let next_victim = self.clocksweep_next_victim.as_ptr();
        // loop {
        //     (*next_victim).state
        // }
    }
}
