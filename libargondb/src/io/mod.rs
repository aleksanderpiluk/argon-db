mod block_cache;
mod io_uring;
mod page_manager;
pub mod read;
mod read_thread;
mod sync;

struct ExecutorIOInterface;

impl ExecutorIOInterface {
    // TODO: Result must be returned
    fn get_block(block_ptr: BlockPointer) -> impl Future<Output = RawBlock> {}
}

struct BlockPointer;

struct RawBlock;
