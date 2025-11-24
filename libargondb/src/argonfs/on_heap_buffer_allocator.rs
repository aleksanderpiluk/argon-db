use std::io::{Cursor, Write};

use crate::argonfs::buffer_allocator::BufferAllocator;

pub struct OnHeapBufferAllocator {
    cursor: Option<Cursor<Box<[u8]>>>,
}

impl OnHeapBufferAllocator {
    fn new() -> Self {
        Self { cursor: None }
    }

    fn into_buffer(self) -> Box<[u8]> {
        match self.cursor {
            None => panic!("attempt of getting unallocated buffer"),
            Some(cursor) => cursor.into_inner(),
        }
    }
}

impl BufferAllocator for OnHeapBufferAllocator {
    fn alloc(&mut self, buf_size: usize) -> &mut dyn Write {
        match self.cursor {
            Some(_) => panic!("attempt of second allocation"),
            None => {
                let buffer = vec![0u8; buf_size].into_boxed_slice();
                let cursor = Cursor::new(buffer);
                self.cursor = Some(cursor);

                let writer_ref: &mut dyn Write = self.cursor.as_mut().unwrap();
                writer_ref
            }
        }
    }
}
