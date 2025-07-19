use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    ptr::NonNull,
};

use super::{
    page::Page,
    page_header::{PageHeader, PageTag},
};

struct BufferPool {
    buffer: NonNull<u8>,
    buffer_layout: Layout,
    headers: NonNull<PageHeader>,
    headers_layout: Layout,
}

impl BufferPool {
    pub fn new(opts: BufferPoolCreationsOpts) -> Self {
        assert!(opts.is_aligned(), "page size must be aligned");

        let buffer_layout = Layout::array::<u8>(opts.buffer_size()).unwrap();
        let ptr = unsafe { alloc(buffer_layout) };
        let buffer = NonNull::new(ptr).unwrap_or_else(|| handle_alloc_error(buffer_layout));

        let headers_layout = Layout::array::<PageHeader>(opts.num_pages).unwrap();
        let ptr = unsafe { alloc(headers_layout) } as *mut PageHeader;
        let headers = NonNull::new(ptr).unwrap_or_else(|| handle_alloc_error(headers_layout));

        Self {
            buffer,
            buffer_layout,
            headers,
            headers_layout,
        }
    }

    pub fn get(tag: PageTag) -> Option<Page> {
        todo!()
    }
}

impl Drop for BufferPool {
    fn drop(&mut self) {
        // TODO: Maybe assertion that all pages are not references is a good idea

        unsafe {
            dealloc(self.buffer, self.buffer_layout);
            dealloc(self.headers, self.headers_layout);
        };
    }
}

struct BufferPoolCreationsOpts {
    page_size: usize,
    num_pages: usize,
}

impl BufferPoolCreationsOpts {
    fn is_aligned(&self) -> bool {
        let alignment = size_of::<libc::max_align_t>();
        self.page_size % alignment == 0
    }

    fn buffer_size(&self) -> usize {
        self.page_size * self.num_pages
    }
}
