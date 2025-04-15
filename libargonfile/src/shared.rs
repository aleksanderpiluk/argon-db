use std::io::{Read, Write};

use anyhow::Result;

use crate::pointer::Pointer;

pub const ARGONFILE_MAGIC: &[u8] = "ARGNFILE".as_bytes();

pub const ARGONFILE_MAGIC_LEN: i64 = 8;
pub const ARGONFILE_TRAILER_LEN: i64 = 12;

pub trait Reader<T> {
    fn try_read<R: Read>(reader: &mut R) -> Result<T>;
}

pub trait Writer<T> {
    fn try_write<W: Write>(writer: &mut PositionedWriter<W>, pointer: &T) -> Result<Pointer>;
}

#[derive(Debug)]
pub struct PositionedWriter<W: Write> {
    writer: W,
    position: usize,
}

impl<W: Write> PositionedWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            position: 0,
        }
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

    pub fn into(self) -> W {
        self.writer
    }
}

impl<W: Write> Write for PositionedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size = self.writer.write(buf)?;
        self.position += size;
        Ok(size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
