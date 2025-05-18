use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
    slice,
    sync::{RwLockReadGuard, RwLockWriteGuard, atomic::Ordering},
};

use super::PageDescriptor;

/// Provides read-write access to a page in the buffer pool.
///
/// Holding a `PageRef` increments the usage count in the corresponding page
/// descriptor. This ensures that page is not replaced.
///
/// When the `PageRef` is dropped, the usage count is automatically decremented.
#[derive(Debug)]
pub struct PageRef<T> {
    descriptor: NonNull<PageDescriptor<T>>,
    page_size: usize,
    ptr: NonNull<u8>,
}

impl<T> PageRef<T> {
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

impl<T> Drop for PageRef<T> {
    fn drop(&mut self) {
        unsafe {
            let ptr_descritor = self.descriptor.as_ptr();
            (*ptr_descritor).usage_count.fetch_sub(1, Ordering::SeqCst); // TODO: Revise memory ordering
        };
    }
}

pub struct PageReadGuard<'a, T> {
    guard: RwLockReadGuard<'a, ()>,
    page: &'a PageRef<T>,
}

impl<'a, T> Deref for PageReadGuard<'a, T> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.page.ptr.as_ptr(), self.page.page_size) }
    }
}

pub struct PageWriteGuard<'a, T> {
    guard: RwLockWriteGuard<'a, ()>,
    page: &'a PageRef<T>,
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
