use std::{
    alloc::{self, Layout},
    cell::UnsafeCell,
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice,
    sync::{
        Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug)]
pub struct BufferPool<T: Default + Eq + Hash> {
    capacity: usize,
    ptr_descriptors: NonNull<PageDescriptor<T>>,
    ptr_pages: NonNull<u8>,
    next_free: UnsafeCell<Option<NonNull<PageDescriptor<T>>>>,
    buffer_table: UnsafeCell<HashMap<T, usize>>,
    lock: Mutex<()>,
}

#[derive(Debug)]
struct PageDescriptor<T> {
    page_tag: T,
    page_id: usize,
    usage_count: AtomicUsize,
    next_free: Option<NonNull<PageDescriptor<T>>>,
    rw_lock: RwLock<()>,
}

impl<T: Default + Eq + Hash> BufferPool<T> {
    const PAGE_SIZE: usize = 8192;

    pub fn new(size: usize) -> Result<Self, ()> {
        let page_total_size = Self::PAGE_SIZE + size_of::<PageDescriptor<T>>();
        let capacity = size / page_total_size;
        assert!(capacity > 0, "Capacity must be greater than 0");

        let layout_pages = Layout::array::<u8>(capacity * Self::PAGE_SIZE).unwrap();
        let layout_descriptors = Layout::array::<PageDescriptor<T>>(capacity).unwrap();

        let ptr_pages = unsafe { alloc::alloc(layout_pages) };
        let ptr_pages = match NonNull::new(ptr_pages) {
            Some(p) => p,
            None => alloc::handle_alloc_error(layout_pages),
        };

        let ptr_descriptors = unsafe { alloc::alloc(layout_descriptors) };
        let ptr_descriptors = match NonNull::new(ptr_descriptors as *mut PageDescriptor<T>) {
            Some(p) => p,
            None => alloc::handle_alloc_error(layout_pages),
        };

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
                usage_count: AtomicUsize::new(0),
                rw_lock: RwLock::new(()),
            }
        };

        Ok(Self {
            capacity,
            ptr_pages,
            ptr_descriptors,
            next_free: UnsafeCell::new(Some(ptr_descriptors)),
            buffer_table: UnsafeCell::new(HashMap::new()),
            lock: Mutex::new(()),
        })
    }

    pub fn try_get_page(&self, page_tag: T) -> Option<Page<T>> {
        None // TODO:
    }

    pub fn get_page(&self, page_tag: T) -> (PageStatus, Page<T>) {
        let lock = self.lock.lock().unwrap();

        let buffer_table = unsafe { self.buffer_table.get().as_mut().unwrap() };
        if let Some(page_id) = buffer_table.get(&page_tag) {
            let descriptor = self.get_page_descriptor(*page_id);
            let page = self.page_for_descriptor(descriptor);
            drop(lock);
            return (PageStatus::EXISTS, page);
        }

        let page = unsafe { self.get_free_page().unwrap() }; // TODO: Add checking of this result

        drop(lock);
        return (PageStatus::INVALID, page);
    }

    unsafe fn get_free_page(&self) -> Option<Page<T>> {
        // let guard = self.lock.lock().unwrap();

        let page = match unsafe { *self.next_free.get() } {
            None => None,
            Some(ptr) => unsafe {
                let next_free = ptr.as_ref().next_free;
                *self.next_free.get() = next_free;

                (*ptr.as_ptr()).next_free = None;
                let page = self.page_for_descriptor(ptr);

                Some(page)
            },
        };

        // drop(guard);

        page
    }

    fn get_page_descriptor(&self, page_id: usize) -> NonNull<PageDescriptor<T>> {
        // TODO: Add range checks
        unsafe { self.ptr_descriptors.add(page_id) }
    }

    fn page_for_descriptor(&self, descriptor: NonNull<PageDescriptor<T>>) -> Page<T> {
        let ptr_descriptor = descriptor.as_ptr();
        unsafe {
            (*ptr_descriptor).usage_count.fetch_add(1, Ordering::SeqCst); // TODO: Revise memory ordering
        }

        let page_id = unsafe { (*ptr_descriptor).page_id };
        Page {
            descriptor,
            page_size: Self::PAGE_SIZE,
            ptr: unsafe { self.ptr_pages.add(page_id * Self::PAGE_SIZE) },
        }
    }
}

impl<T: Default + Eq + Hash> Drop for BufferPool<T> {
    fn drop(&mut self) {
        let layout_pages = Layout::array::<u8>(self.capacity * Self::PAGE_SIZE).unwrap();
        let layout_descriptors = Layout::array::<PageDescriptor<T>>(self.capacity).unwrap();
        unsafe {
            alloc::dealloc(self.ptr_pages.as_ptr(), layout_pages);
            alloc::dealloc(self.ptr_descriptors.as_ptr() as *mut u8, layout_descriptors);
        }
        // let ptr = self.data.as_mut_ptr();
        // unsafe { libc::free(ptr.cast::<c_void>()) };
    }
}

#[derive(Debug)]
pub struct Page<T> {
    descriptor: NonNull<PageDescriptor<T>>,
    page_size: usize,
    ptr: NonNull<u8>,
}

impl<T> Page<T> {
    pub fn read(&self) -> PageReadGuard<T> {
        let ptr_descritor = self.descriptor.as_ptr();
        let guard = unsafe { (*ptr_descritor).rw_lock.read().unwrap() };

        PageReadGuard { page: self, guard }
    }

    pub fn write(&self) -> PageWriteGuard<T> {
        let ptr_descritor = self.descriptor.as_ptr();
        let guard = unsafe { (*ptr_descritor).rw_lock.write().unwrap() };

        PageWriteGuard { page: self, guard }
    }
}

impl<T> Drop for Page<T> {
    fn drop(&mut self) {
        unsafe {
            let ptr_descritor = self.descriptor.as_ptr();
            (*ptr_descritor).usage_count.fetch_sub(1, Ordering::SeqCst); // TODO: Revise memory ordering
        };
    }
}

pub struct PageReadGuard<'a, T> {
    guard: RwLockReadGuard<'a, ()>,
    page: &'a Page<T>,
}

impl<'a, T> Deref for PageReadGuard<'a, T> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.page.ptr.as_ptr(), self.page.page_size) }
    }
}

pub struct PageWriteGuard<'a, T> {
    guard: RwLockWriteGuard<'a, ()>,
    page: &'a Page<T>,
}

impl<'a, T> Deref for PageWriteGuard<'a, T> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.page.ptr.as_ptr(), self.page.page_size) }
    }
}

impl<'a, T> DerefMut for PageWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.page.ptr.as_ptr(), self.page.page_size) }
    }
}

pub enum PageStatus {
    EXISTS,
    INVALID,
}

#[cfg(test)]
mod tests {
    use super::BufferPool;

    #[test]
    #[should_panic]
    fn test_should_panic_with_size_zero() {
        BufferPool::<u64>::new(0).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_should_panic_with_not_sufficient_size() {
        BufferPool::<u64>::new(BufferPool::<u64>::PAGE_SIZE + 1).unwrap();
    }

    #[test]
    fn test_should_create_buffer_pool() {
        let pool_size = BufferPool::<u64>::PAGE_SIZE * 10;

        let pool = BufferPool::<u64>::new(pool_size).unwrap();

        println!("{:?}", pool);
        // assert_eq!(pool.data.len(), pool_size)
    }

    // #[test]
    // fn test_get_free_page() {
    //     let pool_size = BufferPool::PAGE_SIZE * 10;
    //     let pool = BufferPool::new(pool_size).unwrap();

    //     for _ in 0..pool.capacity {
    //         let page = pool.get_free_page().unwrap();
    //         println!("{:?}", page);
    //         println!("{:?}", unsafe { page.descriptor.as_ref() });
    //         println!("{:?}", &page[0..10]);
    //     }
    //     assert!(pool.get_free_page().is_none());
    //     // assert_eq!(pool.data.len(), pool_size)
    // }
}
