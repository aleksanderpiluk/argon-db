use std::{sync::Arc, thread};

use crate::core::error::ExecPoolError;
use crossbeam::deque::Injector;

use super::ExecPoolTask;

const MAX_EXEC_POOL_SIZE: usize = 512;

pub struct ExecPoolCtl {
    task_queue: Arc<Injector<ExecPoolTask>>,
}

impl ExecPoolCtl {
    pub fn init(pool_size: usize) -> Result<Self, ExecPoolError> {
        check_input_pool_size(pool_size)?;

        let task_queue = Arc::new(Injector::new());

        for id in 0..pool_size {
            let task_queue = task_queue.clone();
            thread::spawn(move || loop {
                match task_queue.steal().success() {
                    Some(task) => println!("There is a task! ID: {}", id),
                    None => std::thread::yield_now(),
                };
            });
        }

        Ok(Self { task_queue })
    }

    pub fn schedule_task(&self, task: ExecPoolTask) {
        self.task_queue.push(task);
    }
}

fn check_input_pool_size(pool_size: usize) -> Result<(), ExecPoolError> {
    if pool_size == 0 {
        return Err(ExecPoolError::new(
            "Execution pool size must be greater than 0",
        ));
    }

    if pool_size > MAX_EXEC_POOL_SIZE {
        return Err(ExecPoolError::new(format!(
            "Execution pool size cannot be bigger than {}",
            MAX_EXEC_POOL_SIZE
        )));
    }

    Ok(())
}
