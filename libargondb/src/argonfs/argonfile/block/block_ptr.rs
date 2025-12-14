use bytes::Buf;

use crate::{argonfs::argonfile::ArgonfileDeserializeError, kv::KVSSTableBlockPtr};

use super::{
    error::ArgonfileWriterError,
    utils::{ArgonfileSizeCountingWriter, ArgonfileWrite},
};

pub struct ArgonfileBlockPointer {
    pub offset: u64,
    pub on_disk_size: u32,
}

impl ArgonfileBlockPointer {
    pub const SERIALIZED_SIZE: usize = 12;

    pub fn new(offset: u64, on_disk_size: u32) -> Self {
        Self {
            offset,
            on_disk_size,
        }
    }

    pub fn deserialize(buf: &[u8]) -> Result<ArgonfileBlockPointer, ArgonfileDeserializeError> {
        let offset_bytes = buf[0..8].try_into()?;
        let size_bytes = buf[8..12].try_into()?;

        let offset = u64::from_le_bytes(offset_bytes);
        let on_disk_size = u32::from_le_bytes(size_bytes);

        Ok(ArgonfileBlockPointer {
            offset,
            on_disk_size,
        })
    }

    pub fn serialize(
        w: &mut impl ArgonfileWrite,
        block_ptr: &ArgonfileBlockPointer,
    ) -> Result<usize, ArgonfileWriterError> {
        let mut writer = ArgonfileSizeCountingWriter::new(w);

        writer.write(&u64::to_le_bytes(block_ptr.offset))?;
        writer.write(&u32::to_le_bytes(block_ptr.on_disk_size))?;

        Ok(writer.size())
    }
}

impl From<KVSSTableBlockPtr> for ArgonfileBlockPointer {
    fn from(value: KVSSTableBlockPtr) -> Self {
        Self {
            offset: value.offset(),
            on_disk_size: value.on_disk_size(),
        }
    }
}
