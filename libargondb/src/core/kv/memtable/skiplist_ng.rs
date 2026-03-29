use crossbeam::epoch::{self, Shared};
use crossbeam::epoch::{Atomic, Guard};
use crossbeam::utils::CachePadded;
use std::cmp;
use std::sync::atomic::AtomicU32;
use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    ops::Index,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::kv::base::Comparator;
use crate::kv::memtable::MemtableMutation;
use crate::kv::primary_key::{self, PrimaryKey, PrimaryKeyComparator};

const HEIGHT_BITS: usize = 5;

const MAX_HEIGHT: usize = 1 << HEIGHT_BITS; // ? Are we sure this is max height?

const HEIGHT_MASK: usize = (1 << HEIGHT_BITS) - 1;

const DELETE_MARK_TAG: usize = 1;

pub struct Skiplist {
    schema: primary_key::Schema,
    head: Head,
    rng_state: CachePadded<AtomicU32>,
    max_height: CachePadded<AtomicUsize>,
}

impl Skiplist {
    pub fn new(schema: primary_key::Schema) -> Self {
        todo!()
    }

    pub fn insert(&self, item: MemtableMutation) {
        let key = todo!();

        let guard = &epoch::pin();
        let search_result = self.search_node(key, guard);

        if search_result.found {
            panic!()
        }

        let tower_height = self.tower_height_rng();
        let node = Node::alloc(tower_height, item);
        for i in 0..tower_height {
            let next_ptr: Shared<Node> = search_result.right[i];
            unsafe {
                (*node).tower[i].store(next_ptr, Ordering::Relaxed);
            }
        }

        let node_shared = Shared::<Node>::from(node);
        for i in 0..tower_height {}
    }

    pub fn delete(&self, key: &PrimaryKey) {
        let guard = &epoch::pin();

        let search_result = self.search_node(key, guard);
        let Some(node) = search_result.found_node else {
            return;
        };

        for i in (0..(node.height() - 1)).rev() {
            loop {
                let ptr = node.tower[i].load(Ordering::Relaxed, guard);
                if ptr.tag() == DELETE_MARK_TAG {
                    break;
                }

                let marked_ptr = ptr.with_tag(DELETE_MARK_TAG);
                if node.tower[i]
                    .compare_exchange(ptr, marked_ptr, Ordering::Relaxed, Ordering::Relaxed, guard)
                    .is_ok()
                {
                    break;
                }
            }
        }

        todo!()
    }

    pub fn get(&self, key: &PrimaryKey) {
        let guard = &epoch::pin();

        let search_result = self.search_node(key, guard);
    }

    fn search_node<'a>(&self, key: &PrimaryKey, guard: &'a Guard) -> SearchResult<'a> {
        let comparator = PrimaryKeyComparator::new(&self.schema);

        let mut result = SearchResult {
            found_node: None,
            left: [&self.head.as_tower(); MAX_HEIGHT],
            right: [Shared::null(); MAX_HEIGHT],
        };

        'retry: loop {
            let mut left: &Tower = self.head.as_tower();

            for i in (0..(MAX_HEIGHT - 1)).rev() {
                // Height indexes are 0 based
                let left_next = left[i].load(Ordering::Relaxed, guard); // TODO: Ordering?    

                if left_next.tag() == DELETE_MARK_TAG {
                    continue 'retry;
                }

                let mut right = left_next;
                loop {
                    loop {
                        let Some(node) = (unsafe { right.as_ref() }) else {
                            break;
                        };

                        let right_next = node.tower[i].load(Ordering::Relaxed, guard); // TODO: Ordering?
                        if right_next.tag() != DELETE_MARK_TAG {
                            break;
                        }

                        right = right_next;
                    }

                    let Some(node) = (unsafe { right.as_ref() }) else {
                        break;
                    };

                    let node_key = todo!();
                    if comparator.cmp(key, node_key).unwrap() == cmp::Ordering::Greater {
                        left = &node.tower;
                    } else {
                        break;
                    }
                }

                if left_next != right
                    && left[i]
                        .compare_exchange(
                            left_next,
                            right,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                            guard,
                        )
                        .is_err()
                {
                    continue 'retry;
                }

                result.left[i] = left;
                result.right[i] = right;
            }

            if let Some(node) = unsafe { result.right[0].as_ref() } {
                let node_key = todo!();
                if comparator.eq(key, node_key).unwrap() {
                    result.found_node = Some(node);
                }
            }

            return result;
        }
    }

    fn tower_height_rng(&self) -> usize {
        let mut height: usize;

        loop {
            let rng_state = self.rng_state.load(Ordering::Relaxed);

            let mut x = rng_state;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;

            if let Ok(_) =
                self.rng_state
                    .compare_exchange(rng_state, x, Ordering::Relaxed, Ordering::Relaxed)
            {
                height = ((x & HEIGHT_MASK) + 1) as usize;
                break;
            }
        }

        let mut max_height = self.max_height.load(Ordering::Relaxed);
        height = usize::min(height, usize::max(max_height + 1, 4));
        while height > max_height {
            match self.max_height.compare_exchange(
                max_height,
                height,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_max_height) => max_height = new_max_height,
            };
        }

        height
    }
}

#[repr(C)]
struct Head {
    ptr: [Atomic<Node>; MAX_HEIGHT],
}

impl Head {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn as_tower(&self) -> &Tower {
        let ptr = self as *const Self as *const Tower;
        unsafe { &(*ptr) }
    }
}

#[repr(C)]
struct Tower {
    ptr: [Atomic<Node>; 0],
}

impl Index<usize> for Tower {
    type Output = Atomic<Node>;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*(&self.ptr as *const Atomic<Node>).add(index) }
    }
}

#[repr(C)]
struct Node {
    data: MemtableMutation,
    height_and_ref_count: AtomicUsize,
    tower: Tower,
}

impl Node {
    fn alloc(tower_height: usize, data: MemtableMutation) -> *mut Self {
        let layout = Self::get_layout(tower_height);
        unsafe {
            let ptr = alloc(layout).cast::<Self>();
            if ptr.is_null() {
                handle_alloc_error(layout);
            }

            ptr::write(&raw mut (*ptr).data, data);
            ptr::write(
                &raw mut (*ptr).height_and_ref_count,
                AtomicUsize::new(todo!()),
            );
            ptr::write_bytes(
                (&raw mut (*ptr).tower.ptr).cast::<Atomic<Self>>(),
                0,
                tower_height,
            );

            ptr
        }
    }

    fn dealloc(ptr: *mut Self) {
        let tower_height = todo!();
        let layout = Self::get_layout(tower_height);

        unsafe {
            dealloc(ptr.cast::<u8>(), layout);
        }
    }

    fn height(&self) -> usize {
        self.height_and_ref_count.load(Ordering::Relaxed) & HEIGHT_MASK // ? Is this correct height ?
    }

    fn refs(&self) -> usize {
        self.height_and_ref_count.load(Ordering::Relaxed) & !HEIGHT_MASK >> HEIGHT_BITS
    }

    fn get_layout(tower_height: usize) -> Layout {
        Layout::new::<Self>()
            .extend(Layout::array::<Atomic<Node>>(tower_height).unwrap())
            .unwrap()
            .0
            .pad_to_align()
    }
}

struct SearchResult<'a> {
    found_node: Option<&'a Node>,
    left: [&'a Tower; MAX_HEIGHT],
    right: [Shared<'a, Node>; MAX_HEIGHT],
}
