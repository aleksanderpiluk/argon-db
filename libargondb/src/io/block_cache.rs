use std::{
    alloc::{self, Layout},
    collections::{HashMap, LinkedList, VecDeque},
    hint,
    ptr::NonNull,
    sync::{
        Mutex,
        atomic::{AtomicU32, AtomicUsize, Ordering},
    },
};

type PageTag = u64;

#[derive(Debug)]
pub struct BlockCache {
    

    block_map: BlockMap,

    
}

impl BlockCache {
    /// Allocates a new `BufferPool` for the given buffer size and page size.
    ///
    /// The buffer size must be large enough to fit at least one page along with its
    /// descriptor. This function allocates only as much memory as is needed to fit a
    /// whole number of pages and their descriptors. As a result, the total memory
    /// allocated may be less than the specified buffer size.
    pub fn new(buffer_size: usize, page_size: usize) -> Result<Self, BufferPoolConstructionError> {
        let alignment = size_of::<libc::max_align_t>();
        if page_size % alignment != 0 {
            return Err(BufferPoolConstructionError::PageSizeNotAligned);
        }

        let total_mem_per_page = page_size + size_of::<PageDescriptor>();
        let capacity = buffer_size / total_mem_per_page;
        if capacity == 0 {
            return Err(BufferPoolConstructionError::NotSufficientBufferSize);
        }

        let descriptors_layout = Layout::array::<PageDescriptor>(capacity).unwrap();
        let pages_layout = Layout::array::<u8>(capacity * page_size).unwrap();

        let (pages, descriptors) = unsafe {
            let pages = NonNull::new(alloc::alloc(pages_layout))
                .unwrap_or_else(|| alloc::handle_alloc_error(pages_layout));
            let descriptors = NonNull::new(alloc::alloc(descriptors_layout) as *mut PageDescriptor)
                .unwrap_or_else(|| alloc::handle_alloc_error(descriptors_layout));

            (pages, descriptors)
        };

        let mut free_list = LinkedList::<NonNull<PageDescriptor>>::new();
        for page_id in 0..capacity {
            unsafe {
                let p = descriptors.add(page_id);
                free_list.push_back(p);
                *(p.as_ptr()) = PageDescriptor::new(page_id);
            };
        }

        let free_list = Mutex::new(free_list);

        Ok(Self {
            page_size,
            capacity,

            pages,
            descriptors,

            block_map: BlockMap::new(),

            free_list,
            sweep_next_page_id: AtomicUsize::new(0),
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
    pub fn get_page(
        &self,
        page_tag: &PageTag,
    ) -> Result<(PageStatus, PageRef<T>), BufferPoolPageRetrieveError> {
        let descriptor = self.block_map.get_page(page_tag).unwrap_or_else(|| {});

        // let lock = self.lock.lock().unwrap();

        // if let Some(descriptor) = unsafe { self._internal_table_lookup(&page_tag) } {
        //     let page = self._internal_page_for_descriptor(descriptor);
        //     drop(lock);
        //     return Ok((PageStatus::EXISTS, page));
        // }

        // if let Some(descriptor) = unsafe { self._internal_acquire_unused_page() } {
        //     let page = self._internal_page_for_descriptor(descriptor);
        //     drop(lock);
        //     return Ok((PageStatus::INVALID, page));
        // }

        // drop(lock);
        // return Err(BufferPoolPageRetrieveError::NoPagesAvailable);
    }

    // fn _internal_page_for_descriptor(&self, descriptor: NonNull<PageDescriptor<T>>) -> PageRef<T> {
    //     let ptr_descriptor = descriptor.as_ptr();
    //     unsafe {
    //         (*ptr_descriptor).usage_count.fetch_add(1, Ordering::SeqCst); // TODO: Revise memory ordering
    //     }

    //     let page_id = unsafe { (*ptr_descriptor).page_id };
    //     PageRef {
    //         descriptor,
    //         page_size: self.page_size,
    //         ptr: unsafe { self.ptr_pages.add(page_id * self.page_size) },
    //     }
    // }

    /// Returns a possibly free page from a free list or by running clock sweep algorithm.
    fn get_free_page_candidate(&self) -> NonNull<PageDescriptor> {
        
    }

    /// Puts a page back to a free list. Returns error if page is pinned.
    fn free_page(&self) {}

}

impl Drop for BlockCache {
    fn drop(&mut self) {
        let layout_pages = Layout::array::<u8>(self.capacity * self.page_size).unwrap();
        let layout_descriptors = Layout::array::<PageDescriptor>(self.capacity).unwrap();
        unsafe {
            alloc::dealloc(self.pages.as_ptr(), layout_pages);
            alloc::dealloc(self.descriptors.as_ptr() as *mut u8, layout_descriptors);
        }
    }
}

struct PageDescriptor {
    page_id: usize,
    page_tag: PageTag,

    waker_queue: Mutex<Option<VecDeque<()>>>,

    // Holds flags, ref_count, usage_count and awaiting_count
    state: PageDescriptorState,
}

impl PageDescriptor {
    fn new(page_id: usize) -> Self {
        Self {
            page_id,
            page_tag: PageTag::default(),
            state: PageDescriptorState::new(),
            waker_queue: Mutex::new(None),
        }
    }

    fn pin_page(&self, bump_usage_count: bool) {}
}

struct PageDescriptorState(AtomicU32);

impl PageDescriptorState {
    fn new() -> Self {
        Self(AtomicU32::new(0))
    }

    fn lock(&self) {}

    fn drop_lock(&self, state: u32) {
        self.0.store(state & (!Self::LOCK_BIT), Ordering::Release);
    }

    fn increment_ref_count(&self, increment_usage_count: bool) {
        let mut increment = Self::REF_COUNT_ONE;
        if increment_usage_count {
            increment += Self::USAGE_COUNT_ONE;
        }

        self.0.fetch_add(increment, Ordering::Relaxed);
    }

    fn decrement_usage_count(&self) {
        let old_state: u32 = self.0.fetch_sub(Self::USAGE_COUNT_ONE, Ordering::Relaxed);
        assert!(Self::_usage_count(old_state) >= 0);
    }
}

/// Provides mapping from page tags to page ids. Assumption is that functions
/// provided for this structures are safe without race-conditions.
///
/// TODO: Currently implemented internally as single hashmap guarded by single mutex,
/// which should probably be changed to sharded implementation
#[derive(Debug)]
struct BlockMap {
    inner: ,
}

impl BlockMap {
    fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    fn get_page(&self, page_tag: &PageTag) -> Option<NonNull<PageDescriptor>> {
        let map = self.inner.lock().unwrap();
        map.get(page_tag).map(|ptr| *ptr)

        // TODO: Page pin and bump ref count
    }

    /// Tries to change tag assigned to page, and put entry in block map.
    ///
    /// If page is pinned before acquiring buffer table lock, function returns with
    /// error - the page is used and another candidate for replacement should be
    /// selected.
    ///
    /// If another block map entry for given page tag exists, fuction returns with error.
    fn try_acquire_page(
        &self,
        page_tag: &PageTag,
        descriptor: NonNull<PageDescriptor>,
    ) -> Result<(), ()> {
        let mut map = self.inner.lock().unwrap();
        if let Some(_) = map.get(page_tag) {
            return Err(());
        }

        // TODO: Page pinned check

        let r = map.insert(*page_tag, descriptor);
        assert!(r.is_none());

        Ok(())
    }
}

#[derive(Debug)]
enum BufferPoolConstructionError {
    PageSizeNotAligned,
    NotSufficientBufferSize,
}

#[derive(Debug)]
enum BufferPoolPageRetrieveError {
    NoPagesAvailable,
}

// struct BlockCachePageFuture;

// impl BlockCachePageFuture {
//     fn for_page() -> Self {}
// }

// impl Future for BlockCachePageFuture {
//     fn poll(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//     }
// }

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
