use std::sync::atomic::{AtomicU64, Ordering};

use bitflags::bitflags;

pub struct BlockLock(AtomicU64);

impl BlockLock {
    pub fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub fn try_shared_lock(&self, bump_usage_count: bool) -> Result<(), TrySharedLockError> {
        let old_state = self.load_state();
        let mut new_state = old_state;

        if old_state.check_flag(BlockLockStateFlags::ExclusiveLock) {
            return Err(TrySharedLockError::ExclusiveLockObtained);
        }

        if old_state.ref_count() == u32::MAX {
            panic!("ref_count exhausted"); // FIXME: Better error handling even doe shouldn't happen
        }

        new_state.ref_count_add();
        if bump_usage_count {
            new_state.usage_count_add();
        }

        match self.try_compare_exchange_state(old_state, new_state) {
            Ok(_) => Ok(()),
            Err(_) => Err(TrySharedLockError::StateChanged),
        }
    }

    pub fn drop_shared_lock(&self) {
        // TODO: Maybe some debug assertions a good idea
        self.0.fetch_sub(1, Ordering::AcqRel);
    }

    pub fn try_exclusive_lock(&self) -> Result<(), TryExclusiveLockError> {
        let old_state = self.load_state();
        let mut new_state = old_state;

        if old_state.ref_count() > 0 {
            return Err(TryExclusiveLockError::ShareLocksObtained);
        }

        if old_state.check_flag(BlockLockStateFlags::ExclusiveLock) {
            return Err(TryExclusiveLockError::ExclusiveLockObtained);
        }

        new_state.set_flag(BlockLockStateFlags::ExclusiveLock);

        match self.try_compare_exchange_state(old_state, new_state) {
            Ok(_) => Ok(()),
            Err(_) => Err(TryExclusiveLockError::StateChanged),
        }
    }

    pub(crate) fn drop_exclusive_lock(&self) {
        let old_state = self.load_state();
        let mut new_state = old_state;

        new_state.clear_flag(BlockLockStateFlags::ExclusiveLock);

        self.try_compare_exchange_state(old_state, new_state)
            .expect("state changed while exclusive lock obtained");
    }

    pub fn load_state(&self) -> BlockLockState {
        let state = self.0.load(Ordering::Acquire);

        BlockLockState(state)
    }

    pub fn try_compare_exchange_state(
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

pub enum TryExclusiveLockError {
    ExclusiveLockObtained,
    ShareLocksObtained,
    StateChanged,
}

pub enum TrySharedLockError {
    ExclusiveLockObtained,
    StateChanged,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockLockState(u64);

impl From<u64> for BlockLockState {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl BlockLockState {
    fn ref_count(&self) -> u32 {
        self.0 as u32
    }

    pub fn usage_count(&self) -> u8 {
        (self.0 >> 32) as u8
    }

    fn ref_count_add(&mut self) {
        assert!(self.ref_count() < u32::MAX, "ref_count increment overflow");

        self.0 += 1;
    }

    fn ref_count_take(&mut self) {
        assert!(self.ref_count() > 0, "ref_count decrement overflow");

        self.0 -= 1;
    }

    fn usage_count_add(&mut self) {
        let mut usage_count = self.usage_count();
        if usage_count < 64 {
            usage_count += 1;
        }

        self.set_usage_count(usage_count);
    }

    pub fn usage_count_take(&mut self) -> u8 {
        let mut usage_count = self.usage_count();
        if usage_count > 0 {
            usage_count -= 1;
        }

        self.set_usage_count(usage_count);

        usage_count
    }

    fn set_usage_count(&mut self, usage_count: u8) {
        self.0 = (self.0 & !(0xFF << 32)) | ((usage_count as u64) << 32);
    }

    fn check_flag(&self, flag: BlockLockStateFlags) -> bool {
        self.0 & flag.bits() != 0
    }

    fn set_flag(&mut self, flag: BlockLockStateFlags) {
        assert!(self.check_flag(flag), "flag {:?} already set", flag);

        self.0 |= flag.bits();
    }

    fn clear_flag(&mut self, flag: BlockLockStateFlags) {
        assert!(!self.check_flag(flag), "flag {:?} not set", flag);

        self.0 &= (!flag).bits();
    }
}

bitflags! {

    #[derive(Debug, Clone, Copy)]
    struct BlockLockStateFlags: u64 {
        const ExclusiveLock = 1 << 40;
    }
}
