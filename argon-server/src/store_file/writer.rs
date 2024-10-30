use std::io::Write;

use crate::cell::Cell;

pub struct Writer<W: Write> {
    inner: W,
}

impl<W: Write> Writer<W> {
    pub fn new(writer: W) -> Self {
        Self { inner: writer }
    }

    pub fn write_cell(&mut self, cell: Cell) -> Result<(), std::io::Error> {
        cell.write_to_writer(&mut self.inner)
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.inner.flush()
    }
}
