use std::sync::atomic::{AtomicU64, Ordering};

pub struct MVCCManager {
    read_id: AtomicU64,
    write_id: AtomicU64,
    write_entries: Vec<WriteEntry>,
}

impl Default for MVCCManager {
    fn default() -> Self {
        Self {
            read_id: AtomicU64::new(0),
            write_id: AtomicU64::new(1),
            write_entries: vec![],
        }
    }
}

impl MVCCManager {
    fn begin_write(&self) -> WriteEntry {
        let write_id = self.write_id.fetch_add(1, Ordering::Relaxed);
        let write_entry = WriteEntry { write_id };

        // self.write_entries.push(write_entry);

        write_entry
    }

    fn complete_write(&self, write_entry: WriteEntry) {}

    fn read_id(&self) -> u64 {
        self.read_id.load(Ordering::Relaxed)
    }

    fn write_id(&self) -> u64 {
        self.write_id.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WriteEntry {
    write_id: u64,
}
