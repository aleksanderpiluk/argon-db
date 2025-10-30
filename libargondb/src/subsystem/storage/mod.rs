mod table_loader;

pub struct SubsystemStorage;

impl SubsystemStorage {
    pub fn initialize() -> Self {
        // Create "system" tables and load their data
        // Load other tables
        todo!()
    }
}

// struct SSTableId(u64);
// struct BlockPtr(u64);
// struct BlockSize(u32);
// struct BlockId(SSTableId, BlockPtr, BlockSize);
