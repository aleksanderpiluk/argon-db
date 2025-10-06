use std::sync::Mutex;

pub struct RCU {
    write_lock: Mutex<()>,
}

impl RCU {
    pub fn request_write(&self) -> RCURequestWriteFuture {
        let lock = match self.write_lock.try_lock() {
            Ok(lock) => lock,
            Err(err) => {}
        };
        todo!()
    }
}

struct RCURequestWriteFuture;

impl Future for RCURequestWriteFuture {
    type Output = RCUWriteLockGuard;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
    }
}

struct RCUWriteLockGuard;
