use core::{fmt, slice};
use std::{
    cell::UnsafeCell,
    cmp,
    io::{self, Write},
    sync::atomic::{AtomicUsize, Ordering},
    vec,
};

/// The maximum size of single WAL segment
const SEGMENT_SIZE_MAX: usize = 512 * 1024 * 1024;

/// The default size of single WAL segment.
/// This value can be changed in configuration.
const SEGMENT_SIZE_DEFAULT: usize = 16 * 1024 * 1024;

#[derive(Debug, PartialEq)]
enum SegmentError {
    SegmentSizeExceeded,
}

impl fmt::Display for SegmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::SegmentSizeExceeded => write!(
                f,
                "the provided segment size exceeds maximum size of {} bytes",
                SEGMENT_SIZE_MAX
            ),
        }
    }
}

#[derive(Debug)]
pub struct Segment {
    size: usize,
    cursor: AtomicUsize,
    buffer: UnsafeCell<Box<[u8]>>,
}

impl Segment {
    pub fn new(segment_size: usize) -> Result<Self, SegmentError> {
        if segment_size > SEGMENT_SIZE_MAX {
            return Err(SegmentError::SegmentSizeExceeded);
        }

        Ok(Self {
            size: segment_size,
            cursor: AtomicUsize::new(0),
            buffer: UnsafeCell::new(vec![0u8; segment_size].into_boxed_slice()),
        })
    }

    pub fn allocate<'a>(&'a self, size: usize) -> Option<SegmentEntryWriter<'a>> {
        loop {
            let current = self.cursor.load(Ordering::Relaxed);
            let next: usize = current + size;
            if next > self.size {
                return None;
            }

            if self
                .cursor
                .compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                unsafe {
                    return Some(SegmentEntryWriter {
                        buffer: &mut (self.buffer.get().as_mut().unwrap())[current..next],
                        pos: 0,
                    });
                }
            }
        }
    }
}

struct SegmentEntryWriter<'a> {
    buffer: &'a mut [u8],
    pos: usize,
}

impl<'a> SegmentEntryWriter<'a> {
    pub fn write(mut self, data: &[u8]) -> io::Result<usize> {
        let pos = cmp::min(self.pos, self.buffer.len());

        let n = (&mut self.buffer[pos..]).write(data)?;
        self.pos += n;

        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_new() -> Result<(), SegmentError> {
        Segment::new(SEGMENT_SIZE_DEFAULT)?;
        Ok(())
    }

    #[test]
    fn test_segment_new_with_exceeded_size() {
        let e = Segment::new(SEGMENT_SIZE_MAX + 1).unwrap_err();
        assert_eq!(e, SegmentError::SegmentSizeExceeded);
    }

    #[test]
    fn test_segment_alocate() {
        let segm = Segment::new(10).unwrap();

        let p = segm.allocate(5).unwrap();
        p.write(b"abcde").unwrap();

        let p = segm.allocate(5).unwrap();
        p.write(b"xyzuv").unwrap();

        let p = segm.allocate(1);
        assert_eq!(p.is_none(), true);
    }

    #[test]
    fn test_segment_alocate_too_large() {
        let segm = Segment::new(12).unwrap();

        let p = segm.allocate(6).unwrap();
        p.write(b"abcdef").unwrap();

        let p = segm.allocate(7);
        assert_eq!(p.is_none(), true);
    }
}
