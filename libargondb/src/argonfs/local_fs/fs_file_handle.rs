use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use async_trait::async_trait;
use memmap::MmapOptions;

use crate::argonfs::fs::{FileHandleError, ReadData, ReadOnlyFileHandle, WriteOnlyFileHandle};

pub struct FsReadOnlyFileHandle {
    file: File,
    pos: u64,
    len: u64,
}

impl FsReadOnlyFileHandle {
    pub fn new(path: &PathBuf) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();

        Ok(Self { file, pos: 0, len })
    }
}

#[async_trait]
impl ReadOnlyFileHandle for FsReadOnlyFileHandle {
    async fn read(&mut self, buf_size: usize) -> Result<ReadData, FileHandleError> {
        self.file.seek(SeekFrom::Start(self.pos))?;

        let mut buf = vec![0u8; buf_size];

        self.file.read_exact(&mut buf)?;

        // let map = unsafe {
        //     MmapOptions::new()
        //         .offset(self.pos)
        //         .len(buf_size)
        //         .map(&self.file)
        // }?;

        // let boxed_map: Box<dyn AsRef<[u8]> + Send + Sync> = Box::new(map);
        // let read_data = ReadData::from(boxed_map);
        let boxed_buf: Box<dyn AsRef<[u8]> + Send + Sync> = Box::new(buf);
        let read_data = ReadData::from(boxed_buf);
        Ok(read_data)
    }

    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, FileHandleError> {
        // This function isn't perfect, but should be enough due to file sizes lower than u64/i64 ranges
        let pos_add = |pos: u64, value: i64| -> Result<u64, FileHandleError> {
            if value >= 0 {
                Ok(pos + (value as u64))
            } else {
                let to_sub = (-value) as u64;
                if to_sub > pos {
                    return Err(FileHandleError::SeekError(
                        "Cannot seek before byte 0".into(),
                    ));
                }

                Ok(pos - to_sub)
            }
        };

        match pos {
            SeekFrom::Start(value) => {
                self.pos = value;
            }
            SeekFrom::Current(value) => {
                self.pos = pos_add(self.pos, value)?;
            }
            SeekFrom::End(value) => {
                self.pos = pos_add(self.len, value)?;
            }
        };

        Ok(self.pos)
    }
}

pub struct FsWriteOnlyFileHandle {
    file: File,
}

impl FsWriteOnlyFileHandle {
    pub fn new(path: &PathBuf) -> Result<Self, std::io::Error> {
        let parent_path = path.parent().unwrap();
        std::fs::create_dir_all(parent_path)?;

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        Ok(Self { file })
    }
}

#[async_trait]
impl WriteOnlyFileHandle for FsWriteOnlyFileHandle {}

impl Write for FsWriteOnlyFileHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}
