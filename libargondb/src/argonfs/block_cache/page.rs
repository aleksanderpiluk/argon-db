use std::{mem, ptr::NonNull, task::Waker};

use crate::argonfs::block_cache::BlockTag;

use super::{block_lock::BlockLock, freelist::FreelistNext};

pub struct PageHeader {
    pub lock: BlockLock,
    pub data: NonNull<u8>,
    pub buf_len: usize,
    pub state: PageState,
}

impl PageHeader {
    pub fn is_freelist_item(&self) -> bool {
        if let PageState::FreelistItem { .. } = self.state {
            true
        } else {
            false
        }
    }

    pub fn is_free(&self) -> bool {
        if let PageState::Free = self.state {
            true
        } else {
            false
        }
    }

    pub fn is_acquired(&self) -> bool {
        if let PageState::Acquired { .. } = self.state {
            true
        } else {
            false
        }
    }

    pub fn is_overflow_page(&self) -> bool {
        if let PageState::OverflowPage { .. } = self.state {
            true
        } else {
            false
        }
    }

    pub fn is_loaded_block(&self) -> bool {
        if let PageState::LoadedBlock { .. } = self.state {
            true
        } else {
            false
        }
    }

    pub fn set_state_freelist_item(&mut self, next_free: FreelistNext) {
        assert!(self.is_free());

        self.state = PageState::FreelistItem { next_free };
    }

    pub fn set_state_free_from_freelist_item(&mut self) -> FreelistNext {
        assert!(self.is_freelist_item());

        let PageState::FreelistItem { next_free } = mem::replace(&mut self.state, PageState::Free)
        else {
            unreachable!()
        };

        next_free
    }

    pub fn set_state_free_from_loaded_block(&mut self) -> (BlockTag, Option<NonNull<PageHeader>>) {
        assert!(self.is_loaded_block());

        let PageState::LoadedBlock {
            tag,
            next_overflow_page,
            ..
        } = mem::replace(&mut self.state, PageState::Free)
        else {
            unreachable!()
        };

        (tag, next_overflow_page)
    }

    pub fn set_state_free_from_overflow_page(&mut self) -> Option<NonNull<PageHeader>> {
        assert!(self.is_overflow_page());

        let PageState::OverflowPage {
            next_overflow_page, ..
        } = mem::replace(&mut self.state, PageState::Free)
        else {
            unreachable!();
        };

        next_overflow_page
    }

    pub fn set_state_acquired(&mut self, tag: BlockTag) {
        assert!(self.is_free());

        self.state = PageState::Acquired {
            tag,
            read_dispatched: false,
            wakers: vec![],
            next_overflow_page: None,
        };
    }

    pub fn set_state_loaded_block(&mut self, block_size: usize) -> Vec<Waker> {
        assert!(self.is_acquired());
        assert!(self.is_read_dispatched());

        let PageState::Acquired {
            tag,
            next_overflow_page,
            ..
        } = self.state
        else {
            unreachable!()
        };

        let PageState::Acquired { wakers, .. } = mem::replace(
            &mut self.state,
            PageState::LoadedBlock {
                tag,
                block_size,
                next_overflow_page,
            },
        ) else {
            unreachable!()
        };

        wakers
    }

    pub fn set_state_overflow_page(&mut self, owner: NonNull<PageHeader>) {
        assert!(self.is_free());

        self.state = PageState::OverflowPage {
            owner,
            next_overflow_page: None,
        };
    }

    pub fn set_next_overflow_page(&mut self, page: NonNull<PageHeader>) {
        match &mut self.state {
            PageState::Acquired {
                next_overflow_page, ..
            } => {
                *next_overflow_page = Some(page);
            }
            PageState::OverflowPage {
                next_overflow_page, ..
            } => {
                *next_overflow_page = Some(page);
            }
            _ => panic!("attempt of setting next_over_page in invalid page state"),
        };
    }

    pub fn is_read_dispatched(&self) -> bool {
        if let PageState::Acquired {
            read_dispatched, ..
        } = self.state
        {
            read_dispatched
        } else {
            panic!("page is not in acquired state");
        }
    }

    pub fn set_read_dispatched_flag(&mut self) {
        if let PageState::Acquired {
            read_dispatched, ..
        } = &mut self.state
        {
            assert!(*read_dispatched == false);
            *read_dispatched = true;
        } else {
            panic!("page is not in acquired state");
        }
    }

    pub fn add_waker(&mut self, waker: Waker) {
        if let PageState::Acquired { wakers, .. } = &mut self.state {
            wakers.push(waker);
        } else {
            panic!("page is not in acquired state");
        }
    }

    pub fn next_overflow_page(&self) -> Option<NonNull<PageHeader>> {
        match &self.state {
            PageState::Acquired {
                next_overflow_page, ..
            } => *next_overflow_page,
            PageState::OverflowPage {
                next_overflow_page, ..
            } => *next_overflow_page,
            PageState::LoadedBlock {
                next_overflow_page, ..
            } => *next_overflow_page,
            _ => panic!("attempt of getting next_over_page in invalid page state"),
        }
    }

    pub fn usage_count_take(&mut self) -> u8 {
        let old_state = self.lock.load_state();
        let mut new_state = old_state;

        let usage_count = new_state.usage_count_take();

        self.lock
            .try_compare_exchange_state(old_state, new_state)
            .expect("failed while write access held");

        usage_count
    }

    pub fn usage_count(&self) -> u8 {
        self.lock.load_state().usage_count()
    }

    pub fn block_tag(&self) -> &BlockTag {
        match &self.state {
            PageState::Acquired { tag, .. } => tag,
            PageState::LoadedBlock { tag, .. } => tag,
            _ => panic!("attempt of getting block_tag in invalid page state"),
        }
    }
}

#[derive(Clone)]
pub enum PageState {
    FreelistItem {
        next_free: FreelistNext,
    },
    Free,
    Acquired {
        tag: BlockTag,
        read_dispatched: bool,
        wakers: Vec<Waker>,
        next_overflow_page: Option<NonNull<PageHeader>>,
    },
    LoadedBlock {
        tag: BlockTag,
        block_size: usize,
        next_overflow_page: Option<NonNull<PageHeader>>,
    },
    OverflowPage {
        owner: NonNull<PageHeader>,
        next_overflow_page: Option<NonNull<PageHeader>>,
    },
}
