use std::path::PathBuf;

pub trait PlatformIOAdapter {
    fn read(&self, request: IOFileReaderRequest) -> Result<Box<dyn AsRef<[u8]>>, ()>;
    fn scan_dir(&self, dir: PathBuf);
}

pub struct IOFileReaderRequest {
    pub path: PathBuf,
    pub offset: u64,
    pub size: usize,
}

pub type BoxedPlatformIOAdapter = Box<dyn PlatformIOAdapter + Send + Sync>;
