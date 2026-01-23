use std::io::{self, SeekFrom};

use super::Trailer;
use super::block::BlockPointer;
use crate::argonfs::{
    argonfile::{
        block::{Block, BlockParser},
        error::ArgonfileParseError,
    },
    fs::{FileHandleError, ReadData, ReadOnlyFileHandle},
};

pub struct ArgonfileReader {
    file_handle: Box<dyn ReadOnlyFileHandle>,
}

impl ArgonfileReader {
    pub fn new(file_handle: Box<dyn ReadOnlyFileHandle>) -> Self {
        Self { file_handle }
    }

    pub async fn read_trailer(&mut self) -> Result<Trailer, ArgonfileReaderError> {
        let trailer_size = Trailer::SERIALIZED_SIZE;

        let buf = self
            .file_handle
            .seek_and_read(SeekFrom::End(-(trailer_size as i64)), trailer_size)
            .await?;

        let trailer = Trailer::parse(buf.as_ref())?;
        Ok(trailer)
    }

    pub async fn read_block(
        &mut self,
        block_ptr: &BlockPointer,
    ) -> Result<Block, ArgonfileReaderError> {
        let offset = block_ptr.offset;
        let on_disk_size = block_ptr.on_disk_size as usize;
        self.file_handle.seek(SeekFrom::Start(offset)).await?;
        let buf = self.file_handle.read(on_disk_size).await?;

        let block = BlockParser::parse(buf.as_ref())?;

        Ok(block)
    }
}

#[derive(Debug)]
pub enum ArgonfileReaderError {
    FileHandleError(FileHandleError),
    ArgonfileParseError(ArgonfileParseError),
}

impl From<ArgonfileParseError> for ArgonfileReaderError {
    fn from(value: ArgonfileParseError) -> Self {
        todo!()
    }
}

impl From<FileHandleError> for ArgonfileReaderError {
    fn from(value: FileHandleError) -> Self {
        todo!()
    }
}

impl From<io::Error> for ArgonfileReaderError {
    fn from(value: io::Error) -> Self {
        todo!()
    }
}
