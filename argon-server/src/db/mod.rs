// use crossbeam::channel;
// use op_registry::op_registry;

mod mvcc;
mod op_registry;
mod system_state;

// pub struct Db {
//     channel_mutation: (channel::Sender<()>, channel::Receiver<()>),
//     channel_response: (channel::Sender<()>, channel::Receiver<()>),
//     channel_flush_writer: (channel::Sender<()>, channel::Receiver<()>),
//     channel_schema_manager: (channel::Sender<()>, channel::Receiver<()>),
// }

// impl Db {
//     pub fn execute_op<F>(&self, op: OpDefinition, callback: F) -> u64
//     where
//         F: FnOnce() + Send + Sync + 'static,
//     {
//         let op_id = op_registry().add_op(Box::new(callback));

//         match op {
//             OpDefinition::TableMutation { table, mutation } => {
//                 let (s, _) = &self.channel_mutation;
//                 s.send(()).unwrap();
//             }
//         }

//         op_id
//     }

//     pub fn test_finish_op(&self, op_id: u64) {
//         let (_, op) = op_registry().get(op_id).unwrap();
//         (op.callback)();
//     }
// }

// pub enum OpDefinition {
//     TableMutation {
//         table: String,
//         mutation: TableMutationDefinition,
//     },
//     // TableRead,
// }

// pub enum TableMutationDefinition {
//     RowMutation {
//         row: String,
//         mutations: Vec<RowMutationDefinition>,
//     },
// }

// pub enum RowMutationDefinition {
//     Put {
//         column: String,
//         timestamp: Option<u64>,
//     },

//     Delete {
//         column: String,
//         timestamp: Option<u64>,
//     },
// }

// #[cfg(test)]
// mod tests {
//     use std::thread;

//     use crossbeam::channel::{self, unbounded};

//     use super::*;

//     #[test]
//     fn test() {
//         let db = Db {
//             channel_mutation: unbounded(),
//             channel_response: unbounded(),
//             channel_flush_writer: unbounded(),
//             channel_schema_manager: unbounded(),
//         };

//         thread::spawn(MutationExecutionThread::init());

//         let response_socket = "0.0.0.0:56432";

//         let op_id = db.execute_op(
//             OpDefinition::TableMutation {
//                 table: String::from("test"),
//                 mutation: TableMutationDefinition::RowMutation {
//                     row: String::from("abc_1"),
//                     mutations: vec![],
//                 },
//             },
//             move || {
//                 println!("Execution result: _");
//                 println!("Response Socket: {}", response_socket);
//             },
//         );

//         db.test_finish_op(op_id);
//     }
// }
