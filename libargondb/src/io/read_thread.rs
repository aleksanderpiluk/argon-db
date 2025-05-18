use super::read::ReadQueue;

struct ReadThread;

impl ReadThread {
    fn run<S: ReadThreadStrategy>(ctx: &ReadThreadContext) -> ! {
        S::run(ctx);
    }
}
pub(crate) struct ReadThreadContext {
    pub(crate) read_queue: ReadQueue,
}

pub(crate) trait ReadThreadStrategy {
    fn run(ctx: &ReadThreadContext) -> !;
}
