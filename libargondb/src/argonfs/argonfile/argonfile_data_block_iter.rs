use bytes::Buf;

use crate::kv::KVSSTableDataBlockIter;

pub struct ArgonfileDataBlockIter<B: Buf> {
    buf: B,
}

impl<B: Buf> ArgonfileDataBlockIter<B> {
    pub fn new(buf: B) -> Self {
        todo!()
    }
}

impl<B: Buf> KVSSTableDataBlockIter for ArgonfileDataBlockIter<B> {
    fn next(&mut self) -> Option<Box<dyn crate::kv::KVScanIteratorItem + Send + Sync>> {
        todo!()
    }
}
