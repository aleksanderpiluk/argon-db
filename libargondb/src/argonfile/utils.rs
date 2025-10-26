use std::io::Write;

use crate::argonfile::error::ArgonfileWriterError;

pub fn checked_write<W: Write>(w: &mut W, data: &[u8]) -> Result<usize, ArgonfileWriterError> {
    let size = w.write(data)?;

    if size == data.len() {
        Ok(size)
    } else {
        Err(ArgonfileWriterError::PartialWrite(size))
    }
}

pub fn inner_writer_error_mapper(
    size: usize,
) -> impl FnOnce(ArgonfileWriterError) -> ArgonfileWriterError {
    move |err: ArgonfileWriterError| match err {
        ArgonfileWriterError::PartialWrite(n) => ArgonfileWriterError::PartialWrite(size + n),
        rest => rest,
    }
}
