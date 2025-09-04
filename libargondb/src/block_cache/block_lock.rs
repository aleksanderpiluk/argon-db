use std::sync::atomic::{AtomicU64, Ordering};

use bitflags::bitflags;

pub struct BlockLock(AtomicU64);

impl BlockLock {
    fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    fn try_exclusive_lock(&self) -> Result<(), TryExclusiveLockError> {
        let state = self.acquire_state();
        let mut new_state = state;

        if state.is_exclusive_lock() {
            return Err(TryExclusiveLockError::AlreadyLocked);
        }

        if state.ref_count() > 0 {
            return Err(TryExclusiveLockError::ShareLocksObtained);
        }

        new_state.set_exclusive_lock();

        match self.try_compare_exchange_state(state, new_state) {
            Ok(_) => Ok(()),
            Err(_) => Err(TryExclusiveLockError::StateChanged),
        }
    }

    pub(crate) fn drop_exclusive_lock(&self) {
        let state = self.acquire_state();
        let mut new_state = state;

        new_state.clear_exclusive_lock();

        if let Err(_) = self.try_compare_exchange_state(state, new_state) {
            panic!("state changed while exclusive lock obtained");
        }
    }

    fn try_shared_lock(&self, bump_usage_count: bool) -> Result<(), TrySharedLockError> {
        let state = self.acquire_state();
        let mut new_state = state;

        if state.is_exclusive_lock() {
            return Err(TrySharedLockError::ExclusiveLockObtained);
        }

        if state.ref_count() == u32::MAX {
            return Err(TrySharedLockError::RefCountExhausted);
        }

        new_state.ref_count_incr();
        if bump_usage_count {
            new_state.usage_count_incr();
        }

        match self.try_compare_exchange_state(state, new_state) {
            Ok(_) => Ok(()),
            Err(_) => Err(TrySharedLockError::StateChanged),
        }
    }

    pub(crate) fn drop_shared_lock(&self) {
        // TODO: Maybe some assertion would be a good idea
        self.0.fetch_sub(1, Ordering::AcqRel);
    }

    fn acquire_state(&self) -> BlockLockState {
        let state = self.0.load(Ordering::Acquire);

        BlockLockState(state)
    }

    fn try_compare_exchange_state(
        &self,
        old_state: BlockLockState,
        new_state: BlockLockState,
    ) -> Result<BlockLockState, BlockLockState> {
        match self.0.compare_exchange(
            old_state.0,
            new_state.0,
            Ordering::Release,
            Ordering::Relaxed,
        ) {
            Ok(state) => Ok(state.into()),
            Err(state) => Err(state.into()),
        }
    }
}

enum TryExclusiveLockError {
    AlreadyLocked,
    ShareLocksObtained,
    StateChanged,
}

enum TrySharedLockError {
    ExclusiveLockObtained,
    RefCountExhausted,
    StateChanged,
}

#[derive(Clone, Copy)]
struct BlockLockState(u64);

impl From<u64> for BlockLockState {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl BlockLockState {
    fn ref_count(&self) -> u32 {
        self.0 as u32
    }

    fn usage_count(&self) -> u8 {
        (self.0 >> 32) as u8
    }

    fn ref_count_incr(&mut self) {
        assert!(self.ref_count() < u32::MAX, "ref_count increment overflow");

        self.0 += 1;
    }

    fn ref_count_decr(&mut self) {
        assert!(self.ref_count() > 0, "ref_count decrement overflow");

        self.0 -= 1;
    }

    fn usage_count_incr(&mut self) {
        let mut usage_count = self.usage_count();

        usage_count += 1;

        self.0 = (self.0 & !(0xFF << 40)) | ((usage_count as u64) << 40);
    }

    fn usage_count_decr(&mut self) {
        let mut usage_count = self.usage_count();

        usage_count -= 1;

        self.0 = (self.0 & !(0xFF << 40)) | ((usage_count as u64) << 40);
    }

    fn is_exclusive_lock(&self) -> bool {
        self.0 & BlockLockStateFlags::ExclusiveLock.bits() != 0
    }

    fn set_exclusive_lock(&mut self) {
        assert!(!self.is_exclusive_lock(), "exclusive lock already set");

        self.0 |= BlockLockStateFlags::ExclusiveLock.bits();
    }

    fn clear_exclusive_lock(&mut self) {
        assert!(self.is_exclusive_lock(), "exclusive lock not set");

        self.0 &= (!BlockLockStateFlags::ExclusiveLock).bits();
    }
}

bitflags! {
    struct BlockLockStateFlags: u64 {
        const ExclusiveLock = 1 << 40;
    }
}
