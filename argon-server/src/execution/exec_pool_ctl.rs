use std::thread;

use crate::core::error::ExecPoolError;
use crossbeam::deque::Injector;

pub struct ExecPoolCtl {
    task_queue: Injector<ExecPoolTask>,
}

impl ExecPoolCtl {
    pub fn init(pool_size: usize) -> Result<Self, ExecPoolError> {
        check_input_pool_size(pool_size)?;

        let task_queue = Injector::new();

        for id in 0..pool_size {
            thread::spawn(|| {
                loop {}
                // task_queue.steal().success(
            });
            println!("{}", id);
        }

        Ok(Self { task_queue })
    }

    pub fn schedule_task(&self, task: ExecPoolTask) {
        self.task_queue.push(task);
    }
}

fn check_input_pool_size(pool_size: usize) -> Result<(), ExecPoolError> {
    if pool_size == 0 {
        return Err(ExecPoolError::new("Execution pool size cannot be 0"));
    }

    if pool_size > 1024 {
        return Err(ExecPoolError::new(
            "Execution pool size cannot be bigger than 1024",
        ));
    }

    Ok(())
}

pub struct ExecPoolTask {}
