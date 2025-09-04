use crate::{foundation::block::BlockTag, limits::WAIT_QUEUE_CAPACITY};
use heapless::Vec;
use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    ptr::NonNull,
    slice,
    sync::{
        Mutex,
        atomic::{AtomicU8, AtomicUsize, Ordering},
    },
    task::Waker,
};

pub struct PageHeader {
    mut_state: Mutex<PageHeaderMutState>,
    ref_count: AtomicUsize,
    usage_count: AtomicU8,
}

pub struct PageHeaderMutState {
    next_free: Option<usize>,
    status: PageStatus,
    wait_queue: Vec<Waker, { WAIT_QUEUE_CAPACITY }>,
}

// impl PageHeader {
//     pub fn call_wakers(&self) {
//         let inner = self.inner.lock().unwrap();
//         for waker in inner.wait_queue {
//             waker.wake();
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq)]
pub enum PageStatus {
    FREE,
    ACQUIRED,
    LOADED,
}

#[derive(Clone, Copy)]
pub struct PageWeakRef(&'static PageHeader, &'static [u8]);

impl PageWeakRef {
    pub fn for_page<const T: usize, const U: usize>(
        pages: &'static Pages<T, U>,
        idx: usize,
    ) -> Self {
        let data = pages.pages_buffer.get(idx);
        let header = &pages.headers[idx];

        Self(header, data)
    }

    pub fn new_page_ref(&self) -> PageRef {
        PageRef::new(*self)
    }
}

pub struct PageRef(PageWeakRef);

impl PageRef {
    fn new(weak_ref: PageWeakRef) -> Self {
        Self(weak_ref)
    }
}

impl Clone for PageRef {
    fn clone(&self) -> Self {
        let weak_ref = self.0;
        weak_ref.0.ref_count.fetch_add(1, Ordering::Relaxed);

        Self(weak_ref)
    }
}

impl Drop for PageRef {
    fn drop(&mut self) {
        let weak_ref = self.0;
        weak_ref.0.ref_count.fetch_sub(1, Ordering::Relaxed);
    }
}

pub struct Pages<const PAGE_SIZE: usize, const NUM_PAGES: usize> {
    pages_buffer: PagesBuffer,
    headers: Vec<PageHeader, NUM_PAGES>,
    next_free: Mutex<Option<usize>>,
}

impl<const PAGE_SIZE: usize, const NUM_PAGES: usize> Pages<PAGE_SIZE, NUM_PAGES> {
    pub fn new() -> Self {
        let pages_buffer = PagesBuffer::new(PAGE_SIZE, NUM_PAGES);

        let mut headers = Vec::new();
        for i in 0..NUM_PAGES {
            headers[i] = PageHeader {
                mut_state: Mutex::new(PageHeaderMutState {
                    status: PageStatus::FREE,
                    wait_queue: Vec::new(),
                    next_free: if (i + 1) < NUM_PAGES {
                        Some(i + 1)
                    } else {
                        None
                    },
                }),
                ref_count: AtomicUsize::new(0),
                usage_count: AtomicU8::new(0),
            }
        }
        let next_free = Mutex::new(Some(0));

        Self {
            pages_buffer,
            headers,
            next_free,
        }
    }

    pub fn free_page(&self) -> Option<usize> {
        let mut next_free = self.next_free.lock().unwrap();

        if let Some(idx) = *next_free {
            let mut state = self.headers[idx].mut_state.lock().unwrap();
            assert_eq!((*state).status, PageStatus::FREE);

            *next_free = state.next_free;
            state.next_free = None;

            Some(idx)
        } else {
            None
        }
    }
}

struct PagesBuffer(usize, usize, NonNull<u8>);

unsafe impl Send for PagesBuffer {}
unsafe impl Sync for PagesBuffer {}

impl PagesBuffer {
    fn new(page_size: usize, num_pages: usize) -> Self {
        let alignment = size_of::<libc::max_align_t>();
        assert!(page_size % alignment == 0, "page size must be aligned");

        let buf_size = page_size * num_pages;
        let buf_layout = Layout::array::<u8>(buf_size).unwrap();
        let ptr = unsafe { alloc(buf_layout) };

        Self(
            page_size,
            num_pages,
            NonNull::new(ptr).unwrap_or_else(|| handle_alloc_error(buf_layout)),
        )
    }

    fn get(&self, idx: usize) -> &[u8] {
        let Self(page_size, num_pages, ptr) = self;
        assert!(idx < *num_pages, "idx exceeds num_pages");

        unsafe {
            let ptr = ptr.add(*page_size * idx).as_ptr();
            slice::from_raw_parts(ptr, *page_size)
        }
    }
}

impl Drop for PagesBuffer {
    fn drop(&mut self) {
        let Self(page_size, num_pages, ptr) = self;

        // TODO: Maybe assertion that all pages are not references is a good idea
        let page_size = *page_size;
        let num_pages = *num_pages;
        let buf_ptr = ptr.as_ptr();

        let buf_size = page_size * num_pages;
        let buf_layout = Layout::array::<u8>(buf_size).unwrap();

        unsafe {
            dealloc(buf_ptr, buf_layout);
        };
    }
}
