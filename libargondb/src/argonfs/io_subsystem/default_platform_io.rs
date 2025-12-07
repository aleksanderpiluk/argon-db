use std::{fs::File, path::Path};

use memmap::MmapOptions;

use crate::argonfs::io_subsystem::core::{IOFileReaderRequest, PlatformIOAdapter};

pub struct DefaultPlatformIo {}

impl DefaultPlatformIo {
    pub fn new() -> DefaultPlatformIo {
        Self {}
    }
}

impl PlatformIOAdapter for DefaultPlatformIo {
    fn read(&self, request: IOFileReaderRequest) -> Result<Box<dyn AsRef<[u8]>>, ()> {
        let file = File::open(request.path).unwrap();
        let map = unsafe {
            MmapOptions::new()
                .offset(request.offset)
                .len(request.size)
                .map(&file)
        }
        .unwrap();

        Ok(Box::new(map))
    }

    fn seek_and_read(
        &self,
        request: super::core::IOFileSeekAndReadRequest,
    ) -> Result<Box<dyn AsRef<[u8]>>, ()> {
        todo!()
    }

    fn scan_dir(&self, dir: &Path) -> Result<super::core::ScanDirResult, ()> {
        todo!()
    }

    fn exists(&self, path: &Path) -> Result<bool, ()> {
        todo!()
    }
}
