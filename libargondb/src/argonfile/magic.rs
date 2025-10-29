use std::io::Write;

use crate::argonfile::{
    error::ArgonfileWriterError,
    utils::{ArgonfileWrite, checked_write},
};

pub const ARGONFILE_MAGIC: &'static [u8; 8] = b"ARGNFILE";

pub struct ArgonfileMagicWriter;

impl ArgonfileMagicWriter {
    pub fn write(w: &mut impl ArgonfileWrite) -> Result<usize, ArgonfileWriterError> {
        w.write(ARGONFILE_MAGIC)
    }
}
