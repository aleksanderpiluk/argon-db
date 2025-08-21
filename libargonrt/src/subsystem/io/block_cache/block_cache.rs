use crate::{
    foundation::block::BlockTag,
    subsystem::io::block_cache::pages::{PageRef, PageWeakRef, Pages},
};
use heapless::FnvIndexMap;
use std::sync::Mutex;

pub struct BlockCache<const PAGE_SIZE: usize, const NUM_PAGES: usize> {
    pages_manager: PagesManager<NUM_PAGES>,
    pages: Pages<PAGE_SIZE, NUM_PAGES>,
}

impl<const PAGE_SIZE: usize, const NUM_PAGES: usize> BlockCache<PAGE_SIZE, NUM_PAGES> {
    pub fn new() -> Self {
        let pages = Pages::new();
        let pages_manager = PagesManager::new();

        Self {
            pages,
            pages_manager,
        }
    }

    pub fn get(&'static self, tag: BlockTag) -> Result<PageRef, ()> {
        self.pages_manager.get(&self.pages, &tag)
    }
}

struct PagesManager<const N: usize> {
    map: Mutex<FnvIndexMap<BlockTag, PageWeakRef, N>>,
}

impl<const N: usize> PagesManager<N> {
    fn new() -> Self {
        Self {
            map: Mutex::new(FnvIndexMap::new()),
        }
    }

    fn get<const T: usize, const U: usize>(
        &self,
        pages: &'static Pages<T, U>,
        tag: &BlockTag,
    ) -> Result<PageRef, ()> {
        let map = self.map.lock().unwrap();

        if let Some(page) = map.get(tag) {
            return Ok(page.new_page_ref());
        }

        drop(map);

        self.assign_page(pages, tag)
    }

    fn assign_page<const T: usize, const U: usize>(
        &self,
        pages: &'static Pages<T, U>,
        tag: &BlockTag,
    ) -> Result<PageRef, ()> {
        if let Some(idx) = pages.free_page() {
            let mut map = self.map.lock().unwrap();
            assert!(!map.contains_key(tag));

            let page_weak_ref = PageWeakRef::for_page(pages, idx);
            let page_ref = page_weak_ref.new_page_ref();
            map.insert(*tag, page_weak_ref);

            Ok(page_ref)
        } else {
            todo!("CLOCK SWEEP");
        }
    }
}
