use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread, time, u64,
};

use crate::{
    DbCtx,
    argonfile::ArgonfileBuilder,
    kv::{
        KVColumnFilter, KVFlushPreStats, KVMergeScanIter, KVPrimaryKeyMarker, KVRangeScan,
        KVRangeScanResult, KVSSTable, KVTable, ShadowingIter, primary_key::KVPrimaryKeySchema,
    },
};

const LEVEL_COMPACTION_THRESHOLD: usize = 10;

pub struct SSTableCompactor;

impl SSTableCompactor {
    pub fn new(db_ctx: Arc<DbCtx>) -> SSTableCompactorHandle {
        let close_flag = Arc::new(AtomicBool::new(false));

        let thread_close_flag = close_flag.clone();
        let handle = thread::spawn(move || sstable_compactor_thread(db_ctx, thread_close_flag));

        SSTableCompactorHandle {
            close_flag,
            handles: vec![handle],
        }
    }
}

fn sstable_compactor_thread(db_ctx: Arc<DbCtx>, close_flag: Arc<AtomicBool>) {
    while !close_flag.load(Ordering::SeqCst) {
        scan_tables_for_compaction(&db_ctx);
        thread::sleep(time::Duration::from_secs(15));
    }

    println!("sstable compactor thread finished");
}

fn scan_tables_for_compaction(db_ctx: &Arc<DbCtx>) {
    let tables = db_ctx.catalog.list_tables();

    for table in tables {
        let mut level_bins = BTreeMap::<u64, Vec<Arc<Box<dyn KVSSTable>>>>::new();

        let sstables = table.list_sstables();
        for sstable in sstables {
            let level = sstable.level();

            if !level_bins.contains_key(&level) {
                level_bins.insert(level, vec![]);
            }

            level_bins.get_mut(&level).unwrap().push(sstable);
        }

        for (level, sstables) in level_bins {
            if sstables.len() > LEVEL_COMPACTION_THRESHOLD {
                println!(
                    "[SSTable Compactor] For table {} level {} compaction threshold reached - compacting...",
                    table.table_name, level
                );

                let new_level = if level < u64::MAX {
                    level + 1
                } else {
                    u64::MAX
                };

                smol::block_on(compact_sstables(db_ctx, new_level, table.clone(), sstables));

                println!("[SSTable Compactor] compaction finished");
            }
        }
    }
}

async fn compact_sstables(
    db_ctx: &Arc<DbCtx>,
    new_level: u64,
    table: Arc<KVTable>,
    sstables: Vec<Arc<Box<dyn KVSSTable>>>,
) {
    let pk_schema = KVPrimaryKeySchema::from_table_schema(&table.table_schema);
    let mut merge_iter = KVMergeScanIter::new(pk_schema.clone());

    let mut pre_stats_builder = MergePreStatsBuilder::new();

    for sstable in &sstables {
        pre_stats_builder.add_sstable(sstable.as_ref().as_ref());

        let scan_result = sstable
            .range_scan(&KVRangeScan::new(
                table.table_schema.clone(),
                KVPrimaryKeyMarker::Start,
                KVPrimaryKeyMarker::End,
                KVColumnFilter::All,
            ))
            .await
            .unwrap();

        if let KVRangeScanResult::Iter(iter) = scan_result {
            merge_iter.add_iter(iter);
        }
    }

    let object_id = db_ctx.kv_instance.generate_compacted_sstable_id();
    let writer = db_ctx
        .persistence
        .new_file_writer_for_sstable(&table.table_id, object_id)
        .await
        .unwrap();

    ArgonfileBuilder::flush_iter(
        writer,
        ShadowingIter::new(merge_iter, pk_schema.clone()).await,
        object_id,
        new_level,
        pre_stats_builder.build(),
    )
    .await
    .unwrap();

    let sstable = db_ctx
        .persistence
        .open_sstable(&table.table_id, object_id, &table.table_schema)
        .await
        .unwrap();

    if let Ok(()) = table.replace_compacted_sstables(&sstables, Arc::new(sstable)) {
        let sstable_ids = sstables
            .iter()
            .map(|sstable| sstable.sstable_id())
            .collect();

        db_ctx
            .persistence
            .remove_compacted_sstables(&table.table_id, sstable_ids)
            .await
            .unwrap();

        println!("[SSTable Compactor] old SSTables successfully removed");
    }
}

pub struct SSTableCompactorHandle {
    close_flag: Arc<AtomicBool>,
    handles: Vec<thread::JoinHandle<()>>,
}

impl SSTableCompactorHandle {
    pub fn close(self) {
        self.close_flag.store(true, Ordering::SeqCst);

        for handle in self.handles {
            handle.join().unwrap();
        }
    }
}

struct MergePreStatsBuilder {
    mutation_count: u64,
}

impl MergePreStatsBuilder {
    fn new() -> Self {
        Self { mutation_count: 0 }
    }

    fn add_sstable(&mut self, sstable: &dyn KVSSTable) {
        self.mutation_count += sstable.mutation_count();
    }

    fn build(self) -> KVFlushPreStats {
        KVFlushPreStats {
            mutations_count: self.mutation_count as usize,
        }
    }
}
