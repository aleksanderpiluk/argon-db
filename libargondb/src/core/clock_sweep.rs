use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ClockSweep {
    next_victim: AtomicUsize,
    buffer_size: usize,
}

impl ClockSweep {
    pub fn _internal_next_victim(&self) {
        let mut victim = self.next_victim.fetch_add(1, Ordering::Relaxed);
        if victim >= self.buffer_size {
            victim = victim % self.buffer_size;
        }

        victim
    }
}
