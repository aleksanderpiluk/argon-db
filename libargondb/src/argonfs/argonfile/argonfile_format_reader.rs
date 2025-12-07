// use std::{
//     io::SeekFrom,
//     path::{Path, PathBuf},
//     sync::Arc,
// };

// use bytes::Buf;

// use crate::{
//     argonfs::{
//         core::{ArgonFsFormatReader, ArgonFsFormatReaderError, BufferAllocator},
//         io_subsystem::{IOFileReaderRequest, IOSubsystem},
//     },
//     kv::{KVSSTableBlockPtr, KVSSTableDataBlockIter},
// };

// pub struct ArgonfileFormatReader {
//     io_subsystem: Arc<IOSubsystem>,
//     file_path: PathBuf,
// }

// impl ArgonfileFormatReader {
//     pub fn new(io_subsystem: Arc<IOSubsystem>, p: impl AsRef<Path>) -> Self {
//         let file_path = PathBuf::from(p.as_ref());

//         Self {
//             io_subsystem,
//             file_path,
//         }
//     }
// }

// impl ArgonFsFormatReader for ArgonfileFormatReader {
//     fn read_kv_sstable_stats(&self) -> Result<crate::kv::KVSSTableStats, ArgonFsFormatReaderError> {
//         let io = self.io_subsystem.platform_io_adapter();

//         // let trailer_data = io.seek_and_read(IOFileSeekAndReadRequest {
//         //     path: self.file_path.clone(),
//         //     seek: SeekFrom::End(),
//         //     size: (),
//         // })?;

//         todo!()
//     }

//     fn read_kv_sstable_summary_index(
//         &self,
//     ) -> Result<crate::kv::KVSSTableSummaryIndex, ArgonFsFormatReaderError> {
//         todo!()
//     }

//     fn load_data_block(
//         &self,
//         block_ptr: KVSSTableBlockPtr,
//         allocator: &mut dyn BufferAllocator,
//     ) -> usize {
//         let io_read_request = IOFileReaderRequest {
//             path: self.file_path.clone(),
//             offset: block_ptr.offset(),
//             size: block_ptr.on_disk_size() as _,
//         };

//         let read_buffer: Box<dyn AsRef<[u8]> + 'static> = self
//             .io_subsystem
//             .platform_io_adapter()
//             .read(io_read_request)
//             .unwrap();

//         let buf: &[u8] = read_buffer.as_ref().as_ref();

//         let block_reader = ArgonfileBlockReader::new(ArgonfileNoCompression);
//         block_reader.read(buf, allocator);

//         todo!()
//     }

//     fn get_data_block_iter(
//         &self,
//         data_block: Box<dyn Buf>,
//     ) -> Box<dyn KVSSTableDataBlockIter + Send + Sync> {
//         todo!()
//     }
// }
