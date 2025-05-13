use std::collections::BTreeSet;

struct CatalogSharedState {
    keyspaces: BTreeSet<KeyspaceSchema>,
}

impl CatalogSharedState {
    fn new() -> Self {
        let catalog = Self {
            keyspaces: BTreeSet::new(),
        };

        // catalog.keyspaces.insert(value)

        catalog
    }
}

struct KeyspaceSchema {}

struct TableSchema {
    name: str,
}

struct SystemSchema {}

// impl SystemSchema {
//     fn new() {
//         let tables = vec![TableSchema {
//             name: "tables",
//             columns: [ColumnSchema {
//                 name: "column_name",
//                 column_type: "string",
//             }],
//         }];
//     }
// }

// struct ColumnSchema {
//     name: str,
//     column_type: str,
// }
