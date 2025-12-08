use std::{fs::File, io::SeekFrom, path::PathBuf};

use async_trait::async_trait;
use memmap::MmapOptions;

use crate::platform::io::{FileHandleError, ReadData, ReadOnlyFileHandle};

pub struct FsReadOnlyFileHandle {
    path: PathBuf,
    pos: u64,
}

impl FsReadOnlyFileHandle {
    pub fn new(path: &PathBuf) -> Self {
        Self {
            path: path.clone(),
            pos: 0,
        }
    }
}

#[async_trait]
impl ReadOnlyFileHandle for FsReadOnlyFileHandle {
    async fn read(&self, buf_size: usize) -> Result<ReadData, FileHandleError> {
        let file = File::open(&self.path).unwrap();
        let map = unsafe { MmapOptions::new().offset(self.pos).len(buf_size).map(&file) }?;

        let boxed_map: Box<dyn AsRef<[u8]>> = Box::new(map);
        let read_data = ReadData::from(boxed_map);
        Ok(read_data)
    }

    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileHandleError> {
        todo!()
    }
}
