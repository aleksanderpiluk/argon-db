// use libargonrt::{
//     argonrt_setup,
//     modules::{basic_connector::BasicConnector, storage::DefaultStorageModule},
// };

// fn main() {
//     let mut rt = argonrt_setup();

//     rt.load_module(DefaultStorageModule::new());
//     rt.load_module(BasicConnector::new());
// }

use libargondb::{
    kv::column_type::ColumnTypeCode,
    ops::{
        create_table::{CreateTableOp, CreateTableOpColumn},
        insert::{InsertOp, InsertOpValue},
        select::SelectOp,
    },
};

fn main() {
    let op = CreateTableOp {
        table_name: "test_table".to_owned(),
        columns: vec![
            CreateTableOpColumn {
                column_name: "id".to_owned(),
                column_type: ColumnTypeCode::Bytes,
            },
            CreateTableOpColumn {
                column_name: "first_name".to_owned(),
                column_type: ColumnTypeCode::Bytes,
            },
            CreateTableOpColumn {
                column_name: "last_name".to_owned(),
                column_type: ColumnTypeCode::Bytes,
            },
        ],
        primary_key: vec!["id".to_owned()],
    };
    let table = op.execute().unwrap();

    let op = InsertOp {
        timestamp: 10,
        values: vec![
            InsertOpValue {
                column_name: "first_name".to_owned(),
                value: "John".as_bytes().to_vec().into_boxed_slice(),
            },
            InsertOpValue {
                column_name: "last_name".to_owned(),
                value: "Doe".as_bytes().to_vec().into_boxed_slice(),
            },
            InsertOpValue {
                column_name: "id".to_owned(),
                value: "12".as_bytes().to_vec().into_boxed_slice(),
            },
        ],
    };

    async_io::block_on(async {
        op.execute(&table).await.unwrap();
    });

    println!("{:#?}", table);

    let op = SelectOp {};

    async_io::block_on(async {
        op.execute(&table).await;
    });
}
