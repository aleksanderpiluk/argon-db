struct CommandId(u64);

struct CommandScheduler {}

impl CommandScheduler {
    fn register_command() -> Result<(), RegisterCommandError> {
        todo!()
    }
}

enum RegisterCommandError {
    CommandPoolFull,
}

struct CommandPool {
    pool: Box<[CommandPoolItem]>,
    next_free: Option<usize>,
}

impl CommandPool {
    fn new(pool_size: usize) -> Self {
        let mut pool = Vec::with_capacity(pool_size);

        for idx in 0..pool_size {
            pool[idx] = CommandPoolItem {
                idx,
                state: ItemState::FREE,
                next_free: Some(idx + 1),
                command_id: CommandId(0),
                total_exec_time: 0,
                input_data: (),
            }
        }
        pool[pool_size - 1].next_free = None;

        Self {
            pool: pool.into_boxed_slice(),
            next_free: Some(0),
        }
    }
}

struct CommandPoolItem {
    idx: usize,
    state: ItemState,
    next_free: Option<usize>,
    command_id: CommandId,
    total_exec_time: u64,
    input_data: (), // TODO:
}

enum ItemState {
    FREE,
    USED,
}
