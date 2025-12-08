use std::io::SeekFrom;

use thiserror::Error;

use super::Trailer;
use crate::{
    argonfs::argonfile::{ArgonfileDeserializeError, block_ptr::ArgonfileBlockPointer},
    platform::io::{FileHandleError, ReadOnlyFileHandle},
};

pub struct ArgonfileReader {
    file_handle: Box<dyn ReadOnlyFileHandle>,
}

impl ArgonfileReader {
    pub fn new(file_handle: Box<dyn ReadOnlyFileHandle>) -> Self {
        Self { file_handle }
    }

    pub async fn read_trailer(&mut self) -> Result<Trailer, ArgonfileReaderError> {
        let trailer_len: usize = Trailer::SERIALIZED_SIZE;
        self.file_handle
            .seek(SeekFrom::End(trailer_len as i64))
            .await?;

        let buf = self.file_handle.read(trailer_len).await?;

        let trailer = Trailer::deserialize(buf.as_ref())?;

        Ok(trailer)
    }

    pub async fn read_block(
        &mut self,
        block_ptr: &ArgonfileBlockPointer,
    ) -> Result<(), ArgonfileReaderError> {
        let offset = block_ptr.offset();
        let on_disk_size = block_ptr.on_disk_size() as usize;
        self.file_handle.seek(SeekFrom::Start(offset)).await?;
        let buf = self.file_handle.read(on_disk_size).await?;

        todo!()
    }
}

#[derive(Error, Debug)]
#[error("argonfile reader error {0}")]
pub enum ArgonfileReaderError {
    FileHandleError(#[from] FileHandleError),
    ArgonfileDeserializeError(#[from] ArgonfileDeserializeError),
}
