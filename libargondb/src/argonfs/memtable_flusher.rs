use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::{DbCtx, argonfs::argonfile::ArgonfileBuilder, kv::memtable::KVMemtableFlushRequest};

pub struct ArgonFsMemtableFlusher {}

impl ArgonFsMemtableFlusher {
    pub fn new(db_ctx: Arc<DbCtx>) -> ArgonFsMemtableFlusherHandle {
        let handle = thread::spawn(|| memtable_flusher_thread(db_ctx));

        ArgonFsMemtableFlusherHandle {
            handles: vec![handle],
        }
    }
}

fn memtable_flusher_thread(db_ctx: Arc<DbCtx>) {
    for flush_request in db_ctx.kv_instance.get_memtable_flush_queue_iter() {
        smol::block_on(process_flush_request(&db_ctx, flush_request));
    }

    println!("memtable flusher thread finished");
}

async fn process_flush_request(db_ctx: &DbCtx, request: KVMemtableFlushRequest) {
    let memtable = request.memtable;
    let table = memtable.table();

    println!(
        "flushing memtable[object_id={}] of table {}[table_id={}]",
        memtable.object_id,
        table.table_name,
        table.table_id.as_ref()
    );

    let object_id = memtable.object_id;
    let table_id = &table.table_id;

    while !memtable.is_flush_ready() {
        std::hint::spin_loop();
    }

    if !memtable.is_flush_needed() {
        println!(
            "flush not needed for memtable[object_id={}] of table {}[table_id={}]",
            memtable.object_id,
            table.table_name,
            table_id.as_ref()
        );
        return;
    }

    let writer = db_ctx
        .persistence
        .new_file_writer_for_sstable(table_id, object_id)
        .await
        .unwrap();

    ArgonfileBuilder::flush_memtable(writer, memtable.clone())
        .await
        .unwrap();

    let sstable = db_ctx
        .persistence
        .open_sstable(table_id, object_id, &table.table_schema)
        .await
        .unwrap();

    table.replace_flushed_memtable_with_sstable(memtable.clone(), Arc::new(sstable));
}

pub struct ArgonFsMemtableFlusherHandle {
    handles: Vec<JoinHandle<()>>,
}

impl ArgonFsMemtableFlusherHandle {
    pub fn close(self) {
        for handle in self.handles {
            handle.join().unwrap();
        }
    }
}
