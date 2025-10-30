use std::io::Write;

use super::block_ptr::{ArgonfileBlockPointer, ArgonfileBlockPointerWriter};
use crate::argonfile::{
    error::ArgonfileWriterError,
    utils::{
        ArgonfileSizeCountingWriter, ArgonfileWrite, checked_write, inner_writer_error_mapper,
    },
};

pub struct ArgonfileTrailer {}

pub struct ArgonfileTrailerReader;

impl ArgonfileTrailerReader {
    pub fn read() {
        todo!()
    }
}

pub struct ArgonfileTrailerWriter;

impl ArgonfileTrailerWriter {
    pub fn write(
        w: &mut impl ArgonfileWrite,
        summary_block_ptr: &ArgonfileBlockPointer,
        stats_block_ptr: &ArgonfileBlockPointer,
        compression_type: u16,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(w);

        ArgonfileBlockPointerWriter::write(&mut writer, summary_block_ptr)?;
        ArgonfileBlockPointerWriter::write(&mut writer, stats_block_ptr)?;
        writer.write(&u16::to_le_bytes(compression_type))?;

        Ok(writer.size())
    }
}
