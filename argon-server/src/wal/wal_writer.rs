// use std::sync::Arc;

// use super::wal_state::*;

// pub struct WALWriter {
//     wal_state: Arc<WALState>,
// }

// impl WALWriter {
//     pub fn new(wal_state: Arc<WALState>) -> Self {
//         Self { wal_state }
//     }

//     pub fn write_entry(&self, data: &[u8]) {
//         let size = data.len();

//         loop {
//             let p = self.wal_state.allocating.allocate(size);
//             if let Some(p) = p {
//                 p.write(data).unwrap();
//                 break;
//             }

//             todo!("WAL allocating segment replacement");
//         }
//     }
// }
