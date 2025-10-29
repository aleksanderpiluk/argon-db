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

pub trait ArgonfileWrite {
    fn offset(&self) -> usize;

    fn write(&mut self, buf: &[u8]) -> Result<usize, ArgonfileWriterError>;
}

pub struct ArgonfileOffsetCountingWriteWrapper<W: Write> {
    writer: W,
    offset: usize,
}

impl<W: Write> ArgonfileOffsetCountingWriteWrapper<W> {
    pub fn new(writer: W) -> Self {
        Self { writer, offset: 0 }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W: Write> ArgonfileWrite for ArgonfileOffsetCountingWriteWrapper<W> {
    fn offset(&self) -> usize {
        self.offset
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ArgonfileWriterError> {
        let size = checked_write(&mut self.writer, buf)?;
        self.offset += size;

        Ok(size)
    }
}

pub struct ArgonfileSizeCountingWriter<'a, W: ArgonfileWrite> {
    inner: &'a mut W,
    size: usize,
}

impl<'a, W: ArgonfileWrite> ArgonfileSizeCountingWriter<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            inner: writer,
            size: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<'a, W: ArgonfileWrite> ArgonfileWrite for ArgonfileSizeCountingWriter<'a, W> {
    fn offset(&self) -> usize {
        self.inner.offset()
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ArgonfileWriterError> {
        let write_size = self
            .inner
            .write(buf)
            .map_err(inner_writer_error_mapper(self.size))?;
        self.size += write_size;

        Ok(write_size)
    }
}
