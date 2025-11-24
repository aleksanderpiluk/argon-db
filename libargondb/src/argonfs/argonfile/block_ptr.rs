use std::io::Write;

use super::{
    error::ArgonfileWriterError,
    utils::{
        ArgonfileSizeCountingWriter, ArgonfileWrite, checked_write, inner_writer_error_mapper,
    },
};

pub struct ArgonfileBlockPointer(u64, u32);

impl ArgonfileBlockPointer {
    pub fn new(offset: u64, on_disk_size: u32) -> Self {
        Self(offset, on_disk_size)
    }

    pub fn offset(&self) -> u64 {
        self.0
    }

    pub fn on_disk_size(&self) -> u32 {
        self.1
    }
}

pub struct ArgonfileBlockPointerWriter;

impl ArgonfileBlockPointerWriter {
    pub fn write(
        w: &mut impl ArgonfileWrite,
        block_ptr: &ArgonfileBlockPointer,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(w);

        writer.write(&u64::to_le_bytes(block_ptr.offset()))?;
        writer.write(&u32::to_le_bytes(block_ptr.on_disk_size()))?;

        Ok(writer.size())
    }
}
