use std::time::SystemTime;

pub struct Clock;

impl Clock {
    pub fn timestamp_current() -> u64 {
        let t = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        t.as_millis() as _
    }
}
