use std::{sync::Arc, thread};

use argon_server::routines::MemstoreFlusher;
use argon_server::routines::OperationsExecutor;
use argon_server::routines::ResponseExecutor;
use argon_server::Database;
use argon_server::Shell;

static OPERATION_EXECUTOR_POOL_SIZE: usize = 2;
static RESPONSE_EXECUTOR_POOL_SIZE: usize = 1;
static MEMSTORE_FLUSHER_POOL_SIZE: usize = 2;

fn main() {
    println!("ArgonDB shell");

    // let db = Arc::new(Database::default());

    thread::scope(|s| {
        for _ in 0..OPERATION_EXECUTOR_POOL_SIZE {
            s.spawn(OperationsExecutor::routine);
        }

        for _ in 0..RESPONSE_EXECUTOR_POOL_SIZE {
            s.spawn(ResponseExecutor::routine);
        }

        for _ in 0..MEMSTORE_FLUSHER_POOL_SIZE {
            s.spawn(MemstoreFlusher::routine);
        }

        s.spawn(|| loop {
            let input = Shell::read_line().unwrap();
            Shell::execute_aql(input);
        });
    });
}
