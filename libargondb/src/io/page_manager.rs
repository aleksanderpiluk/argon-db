use std::{
    collections::{HashMap, LinkedList},
    hint,
    ptr::NonNull,
    sync::{
        Mutex,
        atomic::{AtomicU32, AtomicUsize, Ordering},
    },
};

struct PageManager {
    page_size: usize,
    capacity: usize,

    descriptors: NonNull<PageHeader>,
    pages: NonNull<u8>,

    page_map: Mutex<HashMap<PageTag, NonNull<PageHeader>>>,

    free_list: Mutex<LinkedList<NonNull<PageHeader>>>,
    sweep_next_page_id: AtomicUsize,
}

impl PageManager {
    fn get_page(&self, page_tag: PageTag) -> Result<PageRef, ()> {
        if let Some(page_header) = self._page_map_retrieve(page_tag) {
            return Ok(self._create_page_ref_for_pinned_header(page_header));
        }

        let mut victim_header_ptr = self._get_victim_page_header();
        let mut page_map = self.page_map.lock().unwrap();

        if let Some(page_header) = page_map.get(&page_tag) {
            todo!("When doing clocksweep desired page became already allocated");
        }

        let page_header = unsafe { victim_header_ptr.as_mut() };

        let old_page_tag = page_header.reassign_page(page_tag);
        page_map.remove(&old_page_tag);
        page_map.insert(page_tag, victim_header_ptr);

        Ok(self._create_page_ref_for_pinned_header(victim_header_ptr))
    }

    fn _page_map_retrieve(&self, page_tag: PageTag) -> Option<NonNull<PageHeader>> {
        let page_map = self.page_map.lock().unwrap();

        if let Some(header_ptr) = page_map.get(&page_tag) {
            let header = unsafe { header_ptr.as_ref() };
            header.pin_page(true).unwrap();

            return Some(*header_ptr);
        }

        return None;
    }

    fn _get_victim_page_header(&self) -> NonNull<PageHeader> {
        let mut list = self.free_list.lock().unwrap();
        if let Some(page) = list.pop_front() {
            return page;
        }
        drop(list);

        loop {
            let header_ptr = self._clocksweep_next_page_header();
            let header = unsafe { header_ptr.as_ref() };

            let mut state = header.lock_page();

            if PageState::ref_count(state) == 1 {
                state -= PageState::USAGE_COUNT_ONE;
                if PageState::usage_count(state) == 0 {
                    break header_ptr;
                }
            }

            header.unlock_page(state);
        }
    }

    fn _clocksweep_next_page_header(&self) -> NonNull<PageHeader> {
        let page_id_unbounded = self.sweep_next_page_id.fetch_add(1, Ordering::Relaxed);

        let page_id = page_id_unbounded % self.capacity;
        // TODO: try replace atomic with bounded page_id to prevent overflows

        self._page_header_for_id(page_id)
    }

    #[inline]
    fn _page_header_for_id(&self, page_id: usize) -> NonNull<PageHeader> {
        unsafe { self.descriptors.add(page_id) }
    }

    fn _create_page_ref_for_pinned_header(&self, page_header: NonNull<PageHeader>) -> PageRef {
        todo!()
    }
}

struct PageHeader {
    page_id: usize,
    page_tag: PageTag,

    // Holds flags, ref_count, usage_count and awaiting_count
    state: AtomicU32,
}

impl PageHeader {
    fn lock_page(&self) -> u32 {
        loop {
            let old_state = self.state.fetch_or(PageState::LOCK_BIT, Ordering::Acquire);

            if old_state & PageState::LOCK_BIT == 0 {
                break old_state;
            }

            hint::spin_loop();
        }
    }

    fn unlock_page(&self, state: u32) {
        self.state
            .store(state & (!PageState::LOCK_BIT), Ordering::Release);
    }

    fn pin_page(&self, bump_usage_count: bool) -> Result<(), ()> {
        let mut state = self.lock_page();

        if state & PageState::REF_COUNT_MASK == PageState::REF_COUNT_MASK {
            return Err(()); // Ref count is full
        }

        state += PageState::REF_COUNT_ONE;

        if bump_usage_count && PageState::usage_count(state) < PageState::USAGE_COUNT_MAX {
            state += PageState::USAGE_COUNT_ONE;
        }

        self.unlock_page(state);

        Ok(())
    }

    fn unpin_page(&self) {
        let mut state = self.lock_page();

        assert!(PageState::ref_count(state) > 0);
        state -= PageState::REF_COUNT_ONE;

        self.unlock_page(state);
    }

    fn reassign_page(&mut self, new_page_tag: PageTag) -> PageTag {
        let mut state = self.lock_page();

        let old_page_tag = self.page_tag;
        self.page_tag = new_page_tag;

        assert!(PageState::ref_count(state) == 1);
        assert!(PageState::usage_count(state) == 0);
        state &= !PageState::VALID_BIT;

        self.unlock_page(state);

        old_page_tag
    }
}

struct PageState;

impl PageState {
    const REF_COUNT_ONE: u32 = 1;
    const REF_COUNT_BITS: u32 = 16;
    const REF_COUNT_MASK: u32 = (Self::REF_COUNT_ONE << Self::REF_COUNT_BITS) - 1;
    const USAGE_COUNT_SHIFT: u32 = 16;
    const USAGE_COUNT_ONE: u32 = 1 << Self::USAGE_COUNT_SHIFT;
    const USAGE_COUNT_BITS: u32 = 4;
    const USAGE_COUNT_MASK: u32 = (Self::USAGE_COUNT_ONE << Self::USAGE_COUNT_BITS) - 1;
    const USAGE_COUNT_MAX: u32 = 15;

    const LOCK_BIT: u32 = 1 << 31;
    const VALID_BIT: u32 = 1 << 30;
    const READ_IN_PROGRESS_BIT: u32 = 1 << 29;

    fn ref_count(state: u32) -> u32 {
        state & Self::REF_COUNT_MASK
    }

    fn usage_count(state: u32) -> u32 {
        (state & Self::USAGE_COUNT_MASK) >> Self::USAGE_COUNT_SHIFT
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct PageTag(u64);

impl PageTag {
    const INVALID: PageTag = Self(0);
}

struct PageRef {}
