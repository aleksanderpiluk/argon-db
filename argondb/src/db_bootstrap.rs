use std::sync::Arc;

use libargondb::{
    Catalog,
    kv::{KVTable, config::KVConfig},
};

pub fn db_bootstrap() {
    let catalog = Arc::new(Catalog::empty());

    create_argonsys_tables(&catalog);

    create_existing_tables();
}

fn create_argonsys_tables(catalog: &Catalog) {
    let kv_config = KVConfig::default();

    let argonsys_tables_table = KVTable::create(kv_config, get_argonsys_tables_table_schema());
    catalog.add_table("_argonsys_tables", argonsys_tables_table);

    let argonsys_columns_table = KVTable::create(kv_config, get_argonsys_columns_table_schema());
    catalog.add_table("_argonsys_columns", argonsys_columns_table);

    todo!()
}

fn create_existing_tables() {
    todo!()
}

fn create_and_load_table() {
    todo!()
}

fn get_argonsys_tables_table_schema() -> KVTableSchema {
    KVTableSchema {
        columns: vec![KVColumnSchema {
            column_id: 1,
            column_name: "table_name".to_string(),
            column_type: ColumnTypeCode::Bytes,
        }],
    }
}

fn get_argonsys_columns_table_schema() -> KVTableSchema {
    KVTableSchema {
        columns: vec![
            KVColumnSchema {
                column_id: 1,
                column_name: "table_name".to_string(),
                column_type: ColumnTypeCode::Bytes,
            },
            KVColumnSchema {
                column_id: 1,
                column_name: "column_name".to_string(),
                column_type: ColumnTypeCode::Bytes,
            },
            KVColumnSchema {
                column_id: 1,
                column_name: "column_id".to_string(),
                column_type: ColumnTypeCode::Bytes,
            },
        ],
    }
}
