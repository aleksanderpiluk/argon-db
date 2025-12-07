use std::io::Write;

use super::block_ptr::{ArgonfileBlockPointer, ArgonfileBlockPointerWriter};
use super::{
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
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(w);

        ArgonfileBlockPointerWriter::write(&mut writer, summary_block_ptr)?;
        ArgonfileBlockPointerWriter::write(&mut writer, stats_block_ptr)?;

        Ok(writer.size())
    }
}
