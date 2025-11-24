use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use bytes::Buf;

use crate::{
    argonfs::{
        argonfile::{
            block::{ArgonfileBlockReader, BlockHeaderReader},
            compression::ArgonfileNoCompression,
        },
        buffer_allocator::BoxBufferAllocator,
        io_subsystem::{BoxIOSubsystem, IOFileReaderRequest},
        sstable_format_reader::SSTableFormatReader,
    },
    kv::{KVSSTableBlockPtr, KVSSTableDataBlockIter},
};

pub struct ArgonfileFormatReader {
    io_subsystem: Arc<BoxIOSubsystem>,
}

impl ArgonfileFormatReader {
    pub fn new(p: impl AsRef<Path>) -> Self {
        todo!()
    }
}

#[async_trait]
impl SSTableFormatReader for ArgonfileFormatReader {
    async fn load_data_block(
        &self,
        ptr: KVSSTableBlockPtr,
        allocator: BoxBufferAllocator,
    ) -> usize {
        let read_buffer: Box<dyn AsRef<[u8]> + 'static> = self
            .io_subsystem
            .read(IOFileReaderRequest {
                path: todo!(),
                offset: ptr.offset(),
                size: ptr.on_disk_size() as _,
            })
            .await
            .unwrap();

        let buf: &[u8] = read_buffer.as_ref().as_ref();

        let block_reader = ArgonfileBlockReader::new(ArgonfileNoCompression);
        block_reader.read(buf, allocator);

        todo!()
    }

    fn get_data_block_iter(
        &self,
        data_block: Box<dyn Buf>,
    ) -> Box<dyn KVSSTableDataBlockIter + Send + Sync> {
        todo!()
    }
}
