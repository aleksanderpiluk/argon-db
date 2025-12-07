use std::{fmt::Debug, mem, sync::Arc};

use async_trait::async_trait;

use crate::kv::{
    error::KVRuntimeError,
    mutation::KVMutation,
    scan::{KVRangeScan, KVScanIterator, KVScanIteratorItem},
};

use super::scan::KVScannable;

pub struct KVSSTable {
    reader: Arc<Box<dyn KVSSTableReader + Send + Sync>>,
    summary_index: KVSSTableSummaryIndex,
    stats: KVSSTableStats,
}

impl KVSSTable {
    pub async fn from_reader(reader: Arc<Box<dyn KVSSTableReader + Send + Sync>>) -> Self {
        let (stats, summary_index) = reader.read_stats_and_index().await;

        Self {
            reader,
            summary_index,
            stats,
        }
    }
}

impl Debug for KVSSTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KVSSTable").finish()
    }
}

pub struct KVSSTableStats {
    min_row: Box<[u8]>,
    max_row: Box<[u8]>,
}

pub struct KVSSTableSummaryIndex {
    entries: Vec<KVSSTableSummaryIndexEntry>,
}

impl KVSSTableSummaryIndex {
    pub fn get_range_scan_blocks(&self, scan: &KVRangeScan) -> Vec<KVSSTableBlockPtr> {
        todo!()
    }
}

pub struct KVSSTableSummaryIndexEntry {
    min_key: Box<[u8]>,
    pointer: (),
}

#[async_trait]
impl KVScannable for KVSSTable {
    async fn range_scan(
        &self,
        scan: &KVRangeScan,
    ) -> Result<Box<dyn KVScanIterator>, KVRuntimeError> {
        // 1. Create "scan plan" by iterating through always loaded index and take blocks to scan through
        let blocks = self.summary_index.get_range_scan_blocks(scan);
        // 2. Create SSTableScanIter and return it - iterator makes proper scan
        let reader = self.reader.clone();
        let iter = KVSSTableScanIter::new(reader, blocks).await;
        Ok(Box::new(iter))
    }
}

pub struct KVSSTableScanIter {
    reader: Arc<Box<dyn KVSSTableReader + Send + Sync>>,
    blocks: Vec<KVSSTableBlockPtr>,
    next_block_idx: usize,
    current_block_iter: Option<Box<dyn KVSSTableDataBlockIter + Send + Sync>>,
    current_entry: Option<Box<dyn KVScanIteratorItem + Send + Sync>>,
}

impl KVSSTableScanIter {
    async fn new(
        reader: Arc<Box<dyn KVSSTableReader + Send + Sync>>,
        blocks: Vec<KVSSTableBlockPtr>,
    ) -> Self {
        let mut this = Self {
            reader,
            blocks,
            next_block_idx: 0,
            current_block_iter: None,
            current_entry: None,
        };

        // Load first entry on initialization
        this.load_next_iter().await;
        this.load_next_entry().await;

        this
    }

    async fn load_next_iter(&mut self) {
        if let Some(block) = self.blocks.get(self.next_block_idx) {
            self.next_block_idx += 1;

            let next_iter = self.reader.read_data_block(block).await;
            self.current_block_iter = Some(next_iter);
        } else {
            self.current_block_iter = None;
        }
    }

    async fn load_next_entry(&mut self) {
        loop {
            if let Some(iter) = &mut self.current_block_iter {
                if let Some(entry) = iter.next() {
                    self.current_entry = Some(entry);
                    break;
                } else {
                    self.load_next_iter();
                }
            } else {
                self.current_entry = None;
                break;
            }
        }
    }
}

#[async_trait]
impl KVScanIterator for KVSSTableScanIter {
    async fn next_mutation(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>> {
        let entry = mem::take(&mut self.current_entry);
        self.load_next_entry();
        entry
    }

    fn peek_mutation(&self) -> Option<&Box<dyn KVScanIteratorItem + Send + Sync>> {
        if let Some(entry) = &self.current_entry {
            Some(entry)
        } else {
            None
        }
    }
}

#[async_trait]
pub trait KVSSTableReader {
    async fn read_stats_and_index(&self) -> (KVSSTableStats, KVSSTableSummaryIndex);

    async fn read_data_block(
        &self,
        ptr: &KVSSTableBlockPtr,
    ) -> Box<dyn KVSSTableDataBlockIter + Send + Sync>;
}

pub trait KVSSTableDataBlockIter {
    fn next(&mut self) -> Option<Box<dyn KVScanIteratorItem + Send + Sync>>;
}

#[async_trait]
pub trait KVSSTableBuilder {
    /**
     * Add next mutation to builded SSTable file. In caller responsibility is to ensure that next mutations are passed in strict order. If given mutation breaks ordering, implementation should error.
     */
    async fn add_mutation<T: KVMutation + Send + Sync>(&mut self, mutation: &T) -> Result<(), ()>;
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct KVSSTableBlockPtr(u64, u32);

impl KVSSTableBlockPtr {
    pub fn new(offset: u64, on_disk_size: u32) -> Self {
        Self(offset, on_disk_size)
    }

    pub fn offset(&self) -> u64 {
        self.0
    }

    pub fn on_disk_size(&self) -> u32 {
        self.1
    }
}

impl Debug for KVSSTableBlockPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KVSSTableBlockPtr")
            .field("offset", &self.offset())
            .field("on_disk_size", &self.on_disk_size())
            .finish()
    }
}
