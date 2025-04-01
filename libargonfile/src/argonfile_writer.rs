use std::io::{Cursor, Write};

use anyhow::{anyhow, Ok, Result};

use crate::shared;

#[derive(Debug)]
pub struct ArgonfileWriter<W: Write> {
    state: WriterState,
    writer: W,
}

impl<W: Write> ArgonfileWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            state: WriterState::Uninitialized,
            writer,
        }
    }

    pub fn begin(mut self) -> Result<()> {
        self.state = self.state.try_transition(WriterState::Begin)?;
        self.writer.write(shared::ARGONFILE_MAGIC)?;

        Ok(())
    }

    pub fn end(mut self) -> Result<()> {
        self.state = self.state.try_transition(WriterState::End)?;
        self.writer.write(shared::ARGONFILE_MAGIC)?;

        Ok(())
    }

    pub fn write_data_block(mut self) -> Result<()> {
        self.state = self.state.try_transition(WriterState::DataBlocks)?;

        Ok(())
    }

    pub fn write_partition() {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriterState {
    Uninitialized,
    Begin,
    DataBlocks,
    OtherBlocks,
    SummaryBlocks,
    End,
}

impl WriterState {
    fn try_transition(self, state: WriterState) -> Result<WriterState> {
        match (self, state) {
            (WriterState::Uninitialized, WriterState::Begin) => Ok(()),
            (WriterState::Begin, WriterState::DataBlocks) => Ok(()),
            _ => Err(anyhow!("Transition error from {:?} to {:?}", self, state)),
        }?;

        Ok(state)
    }
}
