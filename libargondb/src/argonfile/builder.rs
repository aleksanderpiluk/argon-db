use async_trait::async_trait;

use crate::{
    argonfile::{
        row::ArgonfileRowBuilder, stats::ArgonfileStatsBuilder, summary::ArgonfileSummaryBuilder,
    },
    kv::{KVSStableBuilder, mutation::KVMutation},
};

struct ArgonfileBuilder {
    summary_builder: ArgonfileSummaryBuilder,
    stats_builder: ArgonfileStatsBuilder,

    row_builder: Option<ArgonfileRowBuilder>,
}

// enum ArgonfileBuilderState {}

impl ArgonfileBuilder {
    pub fn begin() -> Result<Self, ()> {
        todo!();

        // 1. Write magic bytes
        // 2. Begin data block writer

        // Ok(Self {})
    }

    pub fn finalize(&mut self) -> Result<(), ()> {
        // 1. Finalize last row/data block
        // 2. Finalize summary block
        // 3. Finalize stats and trailer
        todo!()
    }

    fn get_row_builder(
        /*<W: Write>*/
        &mut self,
        mutation: &impl KVMutation,
        // w: &mut W,
    ) -> &mut ArgonfileRowBuilder {
        todo!()
        // if let Some(row_builder) = self.row_builder
        //     && !row_builder.belongs_to_row(mutation)
        // {
        //     row_builder.end_row(w);
        // }
    }
}

#[async_trait]
impl KVSStableBuilder for ArgonfileBuilder {
    async fn add_mutation<T: KVMutation + Send + Sync>(&mut self, mutation: &T) -> Result<(), ()> {
        // TODO: Check mutation ordering

        let row_builder = self.get_row_builder(mutation);
        row_builder.add_mutation(mutation);

        // Add mutation to data block writer (row writer)
        // Finalize block if necessary
        todo!()
    }
}
