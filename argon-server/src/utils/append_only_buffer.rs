use std::{
    cell::UnsafeCell,
    io::Write,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Concurrent, non-blocking append-only buffer with a fixed size.
struct AppendOnlyBuffer {
    reserved: usize,
    position: AtomicUsize,
    data: UnsafeCell<Box<[u8]>>,
}

impl AppendOnlyBuffer {
    /// Creates an empty `AppendOnlyBuffer` with specified size in bytes.
    fn with_size(size: usize) -> Self {
        Self {
            reserved: size,
            position: AtomicUsize::new(0),
            data: UnsafeCell::new(vec![0u8; size].into_boxed_slice()),
        }
    }

    /// Performs a non-blocking write to the buffer. Internally it is atomically advancing the index pointer to reserve space.
    pub fn write(&self, buf: &[u8]) -> Result<(), AppendOnlyBufferError> {
        let size = buf.len();
        let mut start: usize;
        loop {
            start = self.move_position(size)?;
            if self
                .position
                .compare_exchange(start, start + size, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        unsafe {
            let data = self.data.get().as_mut().unwrap();
            (&mut data[size..(start + size)]).write(buf).unwrap();
        }

        Ok(())
    }

    // Tries to advance index pointer by the size required to reserve buffer slice.
    fn move_position(&self, size: usize) -> Result<usize, AppendOnlyBufferError> {
        let current = self.position.load(Ordering::Relaxed);
        let new = current + size;
        if new > self.reserved {
            return Err(AppendOnlyBufferError::CapacityError);
        }

        Ok(current)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum AppendOnlyBufferError {
    CapacityError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_add_successfully() {
        let buf = AppendOnlyBuffer::with_size(10);

        buf.write(b"abcde").unwrap();
        buf.write(b"xyzuv").unwrap();
    }

    #[test]
    fn should_error_when_capacity_exceeded() {
        let buf = AppendOnlyBuffer::with_size(12);

        buf.write(b"abcdef").unwrap();

        let p = buf.write(b"too long");
        assert_eq!(p.unwrap_err(), AppendOnlyBufferError::CapacityError);
    }
}
