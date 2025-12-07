use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
};

pub trait PlatformIOAdapter {
    fn read(&self, request: IOFileReaderRequest) -> Result<Box<dyn AsRef<[u8]>>, ()>;
    fn seek_and_read(&self, request: IOFileSeekAndReadRequest) -> Result<Box<dyn AsRef<[u8]>>, ()>;
    fn scan_dir(&self, dir: &Path) -> Result<ScanDirResult, ()>;
    fn exists(&self, path: &Path) -> Result<bool, ()>;
}

pub struct IOFileReaderRequest {
    pub path: PathBuf,
    pub offset: u64,
    pub size: usize,
}

pub struct IOFileSeekAndReadRequest {
    pub path: PathBuf,
    pub seek: SeekFrom,
    pub size: usize,
}

pub type BoxedPlatformIOAdapter = Box<dyn PlatformIOAdapter + Send + Sync>;

pub struct ScanDirResult {
    pub dirs: Vec<()>,
    pub files: Vec<PathBuf>,
}
