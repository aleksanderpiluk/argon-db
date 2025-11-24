use async_trait::async_trait;
use memmap::MmapOptions;
use std::{
    any::Any,
    fs::File,
    io::Write,
    ops::{Deref, Index},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::kv::{KVSSTableBlockPtr, KVSSTableDataBlockIter};

// pub struct IOFileReaderRequest {
//     pub path: PathBuf,
//     pub offset: u64,
//     pub size: usize,
// }

// #[async_trait]
// pub trait IOFileReader {
//     async fn read(&self, request: IOFileReaderRequest) -> Result<Box<dyn AsRef<[u8]>>, ()>;
// }

// pub struct MMapIOFileReader;

// #[async_trait]
// impl IOFileReader for MMapIOFileReader {
//     async fn read(&self, request: IOFileReaderRequest) -> Result<Box<dyn AsRef<[u8]>>, ()> {
//         let file = File::open(request.path).unwrap();
//         let map = unsafe {
//             MmapOptions::new()
//                 .offset(request.offset)
//                 .len(request.size)
//                 .map(&file)
//         }
//         .unwrap();

//         Ok(Box::new(map))
//     }
// }
