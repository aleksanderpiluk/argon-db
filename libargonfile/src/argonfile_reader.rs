use std::io::{Read, Seek, SeekFrom};

use crc32c::crc32c;

use crate::shared::{ARGONFILE_MAGIC_LEN, ARGONFILE_TRAILER_LEN};

pub struct ArgonfileReader<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> ArgonfileReader<R> {
    fn new(reader: R) -> Self {
        Self { reader }
    }

    fn try_read_trailer(mut self) {
        self.reader
            .seek(SeekFrom::End(ARGONFILE_TRAILER_LEN + ARGONFILE_MAGIC_LEN));
    }
}

trait BlockChecksumAlgorithm {
    fn verify(data: &[u8], checksum: &[u8]) -> bool;
}

pub struct BlockChecksumCRC32C {}

impl BlockChecksumAlgorithm for BlockChecksumCRC32C {
    fn verify(data: &[u8], checksum: &[u8]) -> bool {
        let checksum_calculated = crc32c(data);
        let checksum = u32::from_be_bytes(checksum.try_into().unwrap());
        checksum_calculated == checksum
    }
}
