struct SSTableId(u64);
struct BlockPtr(u64);
struct BlockSize(u32);
struct BlockId(SSTableId, BlockPtr, BlockSize);
