use libargondb::ArgonfileReader;
use libargondb::FsFileSystem;
use libargondb::FsFileSystemConfig;
use libargondb::argonfile::StatsParser;
use libargondb::argonfile::{ArgonfileDataBlockIter, SummaryParser};
use libargondb::kv::KVSSTableDataBlockIter;
use libargondb::kv::KVTableSchema;
use libargondb::kv::column_type::ColumnTypeCode;
use libargondb::kv::mutation::MutationUtils;
use libargondb::kv::primary_key::KVPrimaryKeyUtils;
use libargondb::kv::schema::KVColumnSchema;
use std::os::linux::raw::stat;
use std::{env, fs, io::Cursor, process};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let schema = KVTableSchema::build(
        vec![
            KVColumnSchema {
                column_id: 1,
                column_name: "id".into(),
                column_type: ColumnTypeCode::Text,
            },
            KVColumnSchema {
                column_id: 2,
                column_name: "value".into(),
                column_type: ColumnTypeCode::U16,
            },
        ],
        vec![1],
    )
    .unwrap();

    let Some(file_path) = args.get(1) else {
        eprintln!("file argument not provided");
        process::exit(1);
    };

    let fs = FsFileSystem::new(FsFileSystemConfig::default());
    let file_handle = fs.get_file_handle(file_path);

    let read_file_handle = file_handle.open_read_only().await.unwrap();
    let mut reader = ArgonfileReader::new(read_file_handle);

    let trailer = reader.read_trailer().await.unwrap();

    println!("TRAILER:");
    println!("SSTable ID: {}", trailer.sstable_id);

    let block = reader.read_block(&trailer.stats_block_ptr).await.unwrap();
    let stats = StatsParser::parse(&block.data).unwrap();

    println!(
        "Min key: {}",
        KVPrimaryKeyUtils::debug_fmt(&schema, &stats.min_row_key).unwrap()
    );
    println!(
        "Max key: {}",
        KVPrimaryKeyUtils::debug_fmt(&schema, &stats.max_row_key).unwrap()
    );

    let block = reader.read_block(&trailer.summary_block_ptr).await.unwrap();
    let summary = SummaryParser::parse(&block.data).unwrap();

    for entry in &summary.entries {
        let block = reader.read_block(&entry.block_ptr).await.unwrap();

        println!(
            "Summary entry key: {}",
            KVPrimaryKeyUtils::debug_fmt(&schema, &entry.key).unwrap()
        );
        println!(
            "BLOCK [checksum_type: {}, compression_type: {}]",
            block.checksum_type, block.compression_type
        );

        let mut iter = ArgonfileDataBlockIter::new(&block.data[..]);
        while let Some(row) = iter.next() {
            println!(
                "{}",
                MutationUtils::debug_fmt(&schema, row.mutation()).unwrap()
            );
        }
    }
}
