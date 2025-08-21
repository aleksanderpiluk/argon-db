use std::slice;

struct SliceRc {
    inner: dyn ManageableMemory,
    data: *const u8,
    len: usize,
}

impl AsRef<[u8]> for SliceRc {
    fn as_ref(&self) -> &[u8] {
        let &Self { data, len, .. } = self;

        unsafe { slice::from_raw_parts(data, len) }
    }
}

impl Clone for SliceRc {
    fn clone(&self) -> Self {}
}

impl Drop for SliceRc {
    fn drop(&mut self) {}
}

trait ManageableMemory {}
