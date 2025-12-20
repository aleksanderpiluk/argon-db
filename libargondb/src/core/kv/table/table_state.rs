use std::{fmt::Debug, mem::replace, sync::Arc};

use crate::kv::{KVRuntimeError, KVRuntimeErrorKind, memtable::Memtable, scan::KVScannable};

#[derive(Clone)]
pub enum KVTableState {
    Active(KVTableStateActive),
    Closed(KVTableStateClosed),
}

impl Debug for KVTableState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KVTableState").finish()
    }
}

impl KVTableState {
    pub fn new_closed(sstables: Vec<Arc<Box<dyn KVScannable>>>) -> Self {
        Self::Closed(KVTableStateClosed {
            read_memtables: vec![],
            sstables,
        })
    }

    pub fn try_as_active(&self) -> Result<&KVTableStateActive, KVRuntimeError> {
        if let Self::Active(state) = self {
            Ok(state)
        } else {
            Err(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::OperationNotAllowed,
                "table state is not active",
            ))
        }
    }

    pub fn try_as_closed(&self) -> Result<&KVTableStateClosed, KVRuntimeError> {
        if let Self::Closed(state) = self {
            Ok(state)
        } else {
            Err(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::OperationNotAllowed,
                "table state is not closed",
            ))
        }
    }

    pub fn list_scannable(&self) -> Result<Vec<&dyn KVScannable>, KVRuntimeError> {
        let active_state = self.try_as_active()?;

        let mut scannable: Vec<&dyn KVScannable> = vec![];

        scannable.push(active_state.current_memtable.as_scannable());

        for memtable in &active_state.read_memtables {
            scannable.push(memtable.as_scannable());
        }

        for sstable in &active_state.sstables {
            let box_ref = sstable.as_ref();
            scannable.push(box_ref.as_ref());
        }

        Ok(scannable)
    }

    pub fn replace_flushed_memtable_with_sstable(
        &self,
        memtable: Arc<Memtable>,
        sstable: Arc<Box<dyn KVScannable>>,
    ) -> Result<KVTableState, KVRuntimeError> {
        match self {
            Self::Active(state) => state.replace_flushed_memtable_with_sstable(memtable, sstable),
            Self::Closed(state) => state.replace_flushed_memtable_with_sstable(memtable, sstable),
        }
    }
}

#[derive(Clone)]
pub struct KVTableStateActive {
    pub current_memtable: Arc<Memtable>,
    pub read_memtables: Vec<Arc<Memtable>>,
    pub sstables: Vec<Arc<Box<dyn KVScannable>>>,
}

impl KVTableStateActive {
    pub fn move_to_closed(&self) -> KVTableState {
        let mut read_memtables = self.read_memtables.clone();

        let memtable_to_flush = self.current_memtable.clone();
        read_memtables.push(self.current_memtable.clone());

        memtable_to_flush.request_flush();

        KVTableState::Closed(KVTableStateClosed {
            read_memtables,
            sstables: self.sstables.clone(),
        })
    }

    pub fn replace_current_memtable(&self, next_memtable: Arc<Memtable>) -> KVTableState {
        let mut next_state = self.clone();

        let memtable_to_flush = replace(&mut next_state.current_memtable, next_memtable);
        next_state.read_memtables.push(memtable_to_flush.clone());

        memtable_to_flush.request_flush();

        KVTableState::Active(next_state)
    }

    pub fn replace_flushed_memtable_with_sstable(
        &self,
        memtable: Arc<Memtable>,
        sstable: Arc<Box<dyn KVScannable>>,
    ) -> Result<KVTableState, KVRuntimeError> {
        let mut next_state = self.clone();

        let memtable_idx = next_state
            .read_memtables
            .iter()
            .position(|this| Arc::ptr_eq(this, &memtable))
            .ok_or(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::OperationNotAllowed,
                "memtable not found in this table state",
            ))?;
        next_state.read_memtables.remove(memtable_idx);

        next_state.sstables.push(sstable);

        Ok(KVTableState::Active(next_state))
    }
}

#[derive(Clone)]
pub struct KVTableStateClosed {
    pub read_memtables: Vec<Arc<Memtable>>,
    pub sstables: Vec<Arc<Box<dyn KVScannable>>>,
}

impl KVTableStateClosed {
    pub fn move_to_active(&self, current_memtable: Arc<Memtable>) -> KVTableState {
        KVTableState::Active(KVTableStateActive {
            current_memtable,
            read_memtables: self.read_memtables.clone(),
            sstables: self.sstables.clone(),
        })
    }

    pub fn replace_flushed_memtable_with_sstable(
        &self,
        memtable: Arc<Memtable>,
        sstable: Arc<Box<dyn KVScannable>>,
    ) -> Result<KVTableState, KVRuntimeError> {
        let mut next_state = self.clone();

        let memtable_idx = next_state
            .read_memtables
            .iter()
            .position(|this| Arc::ptr_eq(this, &memtable))
            .ok_or(KVRuntimeError::with_msg(
                KVRuntimeErrorKind::OperationNotAllowed,
                "memtable not found in this table state",
            ))?;
        next_state.read_memtables.remove(memtable_idx);

        next_state.sstables.push(sstable);

        Ok(KVTableState::Closed(next_state))
    }
}
