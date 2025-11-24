use std::io::Write;

use bytes::Buf;

pub trait BufferAllocator {
    fn alloc(&mut self, buf_size: usize) -> &mut dyn BufferHandle;
}

pub trait BufferHandle {
    fn get_writer(&mut self) -> &mut dyn Write;
    fn get_buf(&mut self) -> &mut dyn Buf;
}
