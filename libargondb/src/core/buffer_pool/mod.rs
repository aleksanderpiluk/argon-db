mod buffer_table;
mod free_list;
mod page_ref;

use std::{
    alloc::{self, Layout},
    hash::Hash,
    ptr::NonNull,
    sync::{
        Mutex, RwLock,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

use buffer_table::BufferTable;

#[derive(Debug)]
pub struct BufferPool<T: Default + Eq + Hash> {
    page_size: usize,
    capacity: usize,
    descriptors: NonNull<PageDescriptor<T>>,
    pages: NonNull<u8>,

    buffer_table: BufferTable<T>,

    lock: Mutex<()>,
}

impl<T: Default + Eq + Hash> BufferPool<T> {
    /// Allocates a new `BufferPool` for the given buffer size and page size.
    ///
    /// The buffer size must be large enough to fit at least one page along with its
    /// descriptor. This function allocates only as much memory as is needed to fit a
    /// whole number of pages and their descriptors. As a result, the total memory
    /// allocated may be less than the specified buffer size.
    pub fn new(buffer_size: usize, page_size: usize) -> Result<Self, BufferPoolConstructionError> {
        let total_page_size = page_size + size_of::<PageDescriptor<T>>();
        let capacity = buffer_size / total_page_size;
        if capacity == 0 {
            return Err(BufferPoolConstructionError::NotSufficientBufferSize);
        }

        let layout_pages = Layout::array::<u8>(capacity * page_size).unwrap();
        let layout_descriptors = Layout::array::<PageDescriptor<T>>(capacity).unwrap();

        let ptr_pages = unsafe { alloc::alloc(layout_pages) };
        let pages = match NonNull::new(ptr_pages) {
            Some(p) => p,
            None => alloc::handle_alloc_error(layout_pages),
        };

        let ptr_descriptors = unsafe { alloc::alloc(layout_descriptors) };
        let descriptors = match NonNull::new(ptr_descriptors as *mut PageDescriptor<T>) {
            Some(p) => p,
            None => alloc::handle_alloc_error(layout_descriptors),
        };

        unsafe { Self::_internal_init_descriptors(descriptors, capacity) };

        Ok(Self {
            page_size,
            capacity,
            pages,
            descriptors,
            // next_free: UnsafeCell::new(Some(ptr_descriptors)),
            buffer_table: BufferTable::new(),
            lock: Mutex::new(()),
        })
    }

    /// Attempts to retrieve a page corresponding to the given `page_tag`.
    ///
    /// If the page already exists in the buffer pool, it is returned with a status of `PageStatus::EXISTS`.
    /// If not, the function tries to acquire a free page or replace unused one from the pool. If successful, the page is
    /// returned with a status of `PageStatus::INVALID`, and must be filled with data by the caller
    /// before use.
    ///
    /// If no unused pages are available and no pages can be replaced, an error is returned.
    ///
    /// # Errors
    ///
    /// Returns `BufferPoolPageRetrieveError::NoPagesAvailable` if there are no free pages and none can be replaced.
    pub fn request_page(
        &self,
        page_tag: T,
    ) -> Result<(PageStatus, PageRef<T>), BufferPoolPageRetrieveError> {
        let lock = self.lock.lock().unwrap();

        if let Some(descriptor) = unsafe { self._internal_table_lookup(&page_tag) } {
            let page = self._internal_page_for_descriptor(descriptor);
            drop(lock);
            return Ok((PageStatus::EXISTS, page));
        }

        if let Some(descriptor) = unsafe { self._internal_acquire_unused_page() } {
            let page = self._internal_page_for_descriptor(descriptor);
            drop(lock);
            return Ok((PageStatus::INVALID, page));
        }

        drop(lock);
        return Err(BufferPoolPageRetrieveError::NoPagesAvailable);
    }

    unsafe fn _internal_init_descriptors(
        ptr_descriptors: NonNull<PageDescriptor<T>>,
        capacity: usize,
    ) {
        for i in 0..(capacity - 1) {
            unsafe {
                let p = ptr_descriptors.add(i);
                *(p.as_ptr()) = PageDescriptor {
                    page_tag: T::default(),
                    page_id: i,
                    next_free: Some(p.add(1)),
                    usage_count: AtomicUsize::new(0),
                    rw_lock: RwLock::new(()),
                }
            };
        }

        unsafe {
            let p = ptr_descriptors.add(capacity - 1).as_ptr();
            *p = PageDescriptor {
                page_tag: T::default(),
                page_id: capacity - 1,
                next_free: None,
                state: AtomicU64::new(0),
                content_lock: RwLock::new(()),
            };
        }
    }

    unsafe fn _internal_table_lookup(&self, page_tag: &T) -> Option<NonNull<PageDescriptor<T>>> {
        let buffer_table = unsafe { self.buffer_table.get().as_mut().unwrap() };
        buffer_table
            .get(page_tag)
            .map(|page_id| unsafe { self.ptr_descriptors.add(*page_id) })
    }

    fn _internal_page_for_descriptor(&self, descriptor: NonNull<PageDescriptor<T>>) -> PageRef<T> {
        let ptr_descriptor = descriptor.as_ptr();
        unsafe {
            (*ptr_descriptor).usage_count.fetch_add(1, Ordering::SeqCst); // TODO: Revise memory ordering
        }

        let page_id = unsafe { (*ptr_descriptor).page_id };
        PageRef {
            descriptor,
            page_size: self.page_size,
            ptr: unsafe { self.ptr_pages.add(page_id * self.page_size) },
        }
    }

    /// Tries to change tag assigned to page, by replacing buffer table entry.
    ///
    /// If page is pinned before acquiring buffer table lock, function returns with
    /// error - the page is used and another candidate for replacement should be
    /// selected.
    fn _replace_page_tag() -> Result<(), ()> {
        Ok(())
    }

    /// Puts a page back to a free list. Returns error if page is pinned.
    fn _free_page() {}
}

impl<T: Default + Eq + Hash> Drop for BufferPool<T> {
    fn drop(&mut self) {
        let layout_pages = Layout::array::<u8>(self.capacity * self.page_size).unwrap();
        let layout_descriptors = Layout::array::<PageDescriptor<T>>(self.capacity).unwrap();
        unsafe {
            alloc::dealloc(self.pages.as_ptr(), layout_pages);
            alloc::dealloc(self.descriptors.as_ptr() as *mut u8, layout_descriptors);
        }
    }
}

#[derive(Debug)]
struct PageDescriptor<T> {
    page_id: usize,
    page_tag: T,

    // Holds flags, ref_count, usage_count and awaiting_count
    state: AtomicU64,

    // Prevents read-write race conditions on page date.
    content_lock: RwLock<()>,
}

#[derive(Debug)]
pub enum PageStatus {
    EXISTS,
    INVALID,
}

#[derive(Debug)]
enum BufferPoolConstructionError {
    NotSufficientBufferSize,
}

#[derive(Debug)]
enum BufferPoolPageRetrieveError {
    NoPagesAvailable,
}

// #[cfg(test)]
// mod tests {
//     use super::BufferPool;

//     #[test]
//     #[should_panic]
//     fn test_should_panic_with_size_zero() {
//         BufferPool::<u64>::new(0, 1024).unwrap();
//     }

//     #[test]
//     #[should_panic]
//     fn test_should_panic_with_not_sufficient_size() {
//         BufferPool::<u64>::new(BufferPool::<u64>::PAGE_SIZE + 1).unwrap();
//     }

//     #[test]
//     fn test_should_create_buffer_pool() {
//         let pool_size = BufferPool::<u64>::PAGE_SIZE * 10;

//         let pool = BufferPool::<u64>::new(pool_size).unwrap();

//         println!("{:?}", pool);
//         // assert_eq!(pool.data.len(), pool_size)
//     }

//     // #[test]
//     // fn test_get_free_page() {
//     //     let pool_size = BufferPool::PAGE_SIZE * 10;
//     //     let pool = BufferPool::new(pool_size).unwrap();

//     //     for _ in 0..pool.capacity {
//     //         let page = pool.get_free_page().unwrap();
//     //         println!("{:?}", page);
//     //         println!("{:?}", unsafe { page.descriptor.as_ref() });
//     //         println!("{:?}", &page[0..10]);
//     //     }
//     //     assert!(pool.get_free_page().is_none());
//     //     // assert_eq!(pool.data.len(), pool_size)
//     // }
// }
