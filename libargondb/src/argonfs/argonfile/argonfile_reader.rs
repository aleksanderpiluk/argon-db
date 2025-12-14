use std::io::SeekFrom;

use thiserror::Error;

use super::Trailer;
use crate::{
    argonfs::argonfile::{
        ArgonfileDeserializeError, block::BlockReader, block_ptr::ArgonfileBlockPointer,
        checksum::ChecksumError, compression::CompressionError, stats::Stats,
        summary::SummaryIndex,
    },
    platform::io::{FileHandleError, ReadData, ReadOnlyFileHandle},
};

pub struct ArgonfileReader {
    file_handle: Box<dyn ReadOnlyFileHandle>,
}

impl ArgonfileReader {
    pub fn new(file_handle: Box<dyn ReadOnlyFileHandle>) -> Self {
        Self { file_handle }
    }

    pub async fn read_trailer(&mut self) -> Result<Trailer, ArgonfileReaderError> {
        self.file_handle
            .seek(SeekFrom::End(Trailer::SERIALIZED_SIZE as _))
            .await?;

        let buf = self.file_handle.read(Trailer::SERIALIZED_SIZE).await?;

        let trailer = Trailer::deserialize(buf.as_ref())?;

        Ok(trailer)
    }

    pub async fn read_summary_index(&mut self) -> Result<SummaryIndex, ArgonfileReaderError> {}

    pub async fn read_stats(&mut self) -> Result<Stats, ArgonfileReaderError> {}

    // pub async fn read_summary_block(
    //     &mut self,
    //     block_ptr: &ArgonfileBlockPointer,
    // ) -> Result<KVSSTableSummaryIndex, ArgonfileReaderError> {
    //     let buf = self.read_block(block_ptr).await?;

    //     let summary_index = Summary::deserialize(buf.as_ref())?;

    //     Ok(summary_index.into())
    // }

    // pub async fn read_stats_block(
    //     &mut self,
    //     block_ptr: &ArgonfileBlockPointer,
    // ) -> Result<KVSSTableStats, ArgonfileReaderError> {
    //     let buf = self.read_block(block_ptr).await?;

    //     let stats = Stats::deserialize(buf.as_ref())?;

    //     Ok(stats.into())
    // }

    pub async fn read_block(
        &mut self,
        block_ptr: &ArgonfileBlockPointer,
    ) -> Result<ReadData, ArgonfileReaderError> {
        let offset = block_ptr.offset;
        let on_disk_size = block_ptr.on_disk_size as usize;
        self.file_handle.seek(SeekFrom::Start(offset)).await?;
        let buf = self.file_handle.read(on_disk_size).await?;

        let block = BlockReader.read(buf.as_ref())?;

        Ok(buf)
    }
}

#[derive(Error, Debug)]
#[error("argonfile reader error {0}")]
pub enum ArgonfileReaderError {
    FileHandleError(#[from] FileHandleError),
    ArgonfileDeserializeError(#[from] ArgonfileDeserializeError),
    CompressionError(#[from] CompressionError),
    ChecksumError(#[from] ChecksumError),
}
