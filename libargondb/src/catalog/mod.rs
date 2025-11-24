mod catalog;
mod catalog_state;

pub use catalog::Catalog;

// struct DatabaseDeclaration {
//     database_name: String,
//     tables: Box<[TableDeclaration]>,
// }

// struct TableDeclaration {
//     table_name: String,
//     primary_key: PrimaryKeySchema,
//     columns: Box<[ColumnDeclaration]>,
// }

// pub struct PrimaryKeySchema {
//     columns: Box<[u16]>,
// }

// struct ColumnDeclaration {
//     column_id: u16,
//     column_name: String,
// }

// fn get_system_database_declaration() -> DatabaseDeclaration {
//     DatabaseDeclaration {
//         database_name: "_system".to_string(),
//         tables: vec![
//             TableDeclaration {
//                 table_name: "tables".to_string(),
//                 primary_key: PrimaryKeySchema {
//                     columns: vec![].into_boxed_slice(),
//                 },
//                 columns: vec![
//                     ColumnDeclaration {
//                         column_id: 0,
//                         column_name: "database_name".to_string(),
//                     },
//                     ColumnDeclaration {
//                         column_id: 1,
//                         column_name: "table_name".to_string(),
//                     },
//                 ]
//                 .into_boxed_slice(),
//             },
//             TableDeclaration {
//                 table_name: "columns".to_string(),
//                 primary_key: PrimaryKeySchema {
//                     columns: vec![].into_boxed_slice(),
//                 },
//                 columns: vec![
//                     ColumnDeclaration {
//                         column_id: 0,
//                         column_name: "database_name".to_string(),
//                     },
//                     ColumnDeclaration {
//                         column_id: 1,
//                         column_name: "table_name".to_string(),
//                     },
//                     ColumnDeclaration {
//                         column_id: 2,
//                         column_name: "column_name".to_string(),
//                     },
//                     ColumnDeclaration {
//                         column_id: 3,
//                         column_name: "column_id".to_string(),
//                     },
//                 ]
//                 .into_boxed_slice(),
//             },
//         ]
//         .into_boxed_slice(),
//     }
// }

// fn catalog_bootstrap() {
//     let db_declarations = vec![get_system_database_declaration()];
// }
