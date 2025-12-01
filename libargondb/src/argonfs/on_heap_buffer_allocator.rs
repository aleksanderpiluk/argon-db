// use std::io::{Cursor, Write};

// use crate::argonfs::buffer_allocator::{BufferAllocator, BufferHandle};

// pub struct OnHeapBufferAllocator {
//     cursor: Option<Box<[u8]>>,
// }

// impl OnHeapBufferAllocator {
//     fn new() -> Self {
//         Self { cursor: None }
//     }

//     fn into_buffer(self) -> Box<[u8]> {
//         match self.cursor {
//             None => panic!("attempt of getting unallocated buffer"),
//             Some(buf) => buf,
//         }
//     }
// }

// impl BufferAllocator for OnHeapBufferAllocator {
//     fn alloc(&mut self, buf_size: usize) -> impl BufferHandle {
//         match self.cursor {
//             Some(_) => panic!("attempt of second allocation"),
//             None => {
//                 let buffer = vec![0u8; buf_size].into_boxed_slice();
//                 // let cursor = Cursor::new(buffer);
//                 self.cursor = Some(buffer);
//                 // self.cursor = Some(cursor);

//                 HeapBufferHandle::new(self.cursor.as_mut().unwrap().as_mut())
//             }
//         }
//     }
// }

// struct HeapBufferHandle<'a> {
//     buf: &'a mut [u8],
// }

// impl<'a> HeapBufferHandle<'a> {
//     fn new(buf: &'a mut [u8]) -> Self {
//         Self { buf }
//     }
// }

// impl<'a> BufferHandle for HeapBufferHandle<'a> {
//     fn get_writer(&mut self) -> &mut dyn Write {
//         &mut self.buf
//     }

//     fn get_buf(&mut self) -> &mut dyn bytes::Buf {
//         todo!()
//     }
// }
