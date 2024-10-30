use std::{cell::UnsafeCell, future::Future, task::Poll};

pub struct ExecPoolTask {
    //     cmd: ExecPoolTaskCmd,
    //     result: UnsafeCell<Option<ExecPoolTaskResult>>,
}

// type ExecPoolTaskResult = Vec<u8>;

// impl ExecPoolTask {
//     pub fn new() -> Self {
//         Self {
//             cmd: ExecPoolTaskCmd::NoOp,
//             result: UnsafeCell::new(None),
//         }
//     }
// }

// impl Future for ExecPoolTask {
//     type Output = Vec<u8>;

//     fn poll(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         match self.result.into_inner() {
//             ExecPoolTaskResult::NoResult => Poll::Pending,
//             ExecPoolTaskResult::Done(res) => Poll::Ready(res.clone()),
//         }
//     }
// }

enum ExecPoolTaskCmd {
    // MutateRow {
    //     table: String,
    //     row: String,
    //     ops: Vec<MutateRowOp>,
    // },
    // MutateRows {
    //     table: String,
    //     mutations: Vec<MutateRowsEntry>,
    // },
    NoOp,
}

// pub struct MutateRowsEntry {
//     row: String,
//     ops: Vec<MutateRowOp>,
// }

// pub enum MutateRowOp {
//     PutCell,
//     DeleteCell,
//     DeleteColumn,
//     DeleteFamily,
// }
