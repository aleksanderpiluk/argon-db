use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    cmp,
    fmt::Debug,
    marker::PhantomData,
    mem::swap,
    ops::{Bound, Deref, Index},
    ptr::{self, NonNull},
    sync::atomic::{self, AtomicUsize, Ordering},
};

use crossbeam::{
    epoch::{Atomic, Guard, Shared, pin},
    utils::CachePadded,
};

use crate::kv::{
    mutation::{KVMutation, MutationComparator, StructuredMutation},
    primary_key::{KVPrimaryKeyComparator, KVPrimaryKeySchema},
};

const HEIGHT_BITS: usize = 5;

const MAX_HEIGHT: usize = 1 << HEIGHT_BITS;

const HEIGHT_MASK: usize = (1 << HEIGHT_BITS) - 1;

const DELETE_MARK_TAG: usize = 1;

pub struct Skiplist {
    schema: KVPrimaryKeySchema,
    height: CachePadded<AtomicUsize>,
    len: CachePadded<AtomicUsize>,
    rng_state: CachePadded<AtomicUsize>,
    head: Head,
}

pub struct Entry<'a> {
    parent: &'a Skiplist,
    node: NodeRef<'a>,
}

pub struct Iter<'a> {
    parent: &'a Skiplist,
    from: Option<Box<[u8]>>,
    to: Option<Box<[u8]>>,
    current: Option<Entry<'a>>,
}

#[repr(C)]
struct Head {
    ptrs: [Atomic<Node>; MAX_HEIGHT],
}

#[repr(C)]
struct Tower {
    ptrs: [Atomic<Node>; 0],
}

#[repr(C)]
struct Node {
    data: StructuredMutation,
    height_refcount: AtomicUsize,
    tower: Tower,
}

struct SearchResult<'a> {
    found_node: Option<NodeRef<'a>>,
    left: [TowerRef<'a>; MAX_HEIGHT],
    right: [Shared<'a, Node>; MAX_HEIGHT],
}

impl Skiplist {
    pub fn new(schema: KVPrimaryKeySchema) -> Self {
        Self {
            schema,
            height: CachePadded::new(AtomicUsize::new(0)),
            len: CachePadded::new(AtomicUsize::new(0)),
            rng_state: CachePadded::new(AtomicUsize::new(0)),
            head: Head::new(),
        }
    }

    pub fn insert(&self, mutation: StructuredMutation) {
        let tower_height = self.tower_height_rng();
        let node_shared = Shared::<Node>::from(Node::alloc(tower_height, 2, mutation));
        let node_ref = NodeRef::from_shared(node_shared).unwrap();

        let guard = &pin();
        let mut search_result = self.search_node(&node_ref.data, guard);

        self.len.fetch_add(1, Ordering::Relaxed);

        loop {
            node_ref.tower[0].store(search_result.right[0], Ordering::Relaxed);

            if search_result.left[0][0]
                .compare_exchange(
                    search_result.right[0],
                    node_shared,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                    guard,
                )
                .is_ok()
            {
                // Node successfully added to level 0 list
                if let Some(old_node) = search_result.found_node {
                    if old_node.mark_tower(guard) {
                        self.len.fetch_sub(1, Ordering::Relaxed);
                    }
                }
                break;
            }

            search_result = self.search_node(&node_ref.data, guard);
        }

        let entry = EntryRef {
            parent: self,
            node: node_ref,
        };

        'build: for level in 1..tower_height {
            loop {
                let next = node_ref.tower[level].load(Ordering::SeqCst, guard);

                if next.tag() == DELETE_MARK_TAG {
                    break 'build;
                }

                if unsafe { search_result.right[level].as_ref() }.is_some_and(|right_node| {
                    MutationComparator::eq(&self.schema, &right_node.data, &entry.node.data)
                        .unwrap()
                }) {
                    search_result = self.search_node(&entry.node.data, guard);
                    continue;
                }

                if node_ref.tower[level]
                    .compare_exchange(
                        next,
                        search_result.right[level],
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                        guard,
                    )
                    .is_err()
                {
                    break 'build;
                }

                node_ref
                    .height_refcount
                    .fetch_add(1 << HEIGHT_BITS, Ordering::Relaxed);

                if search_result.left[level][level]
                    .compare_exchange(
                        search_result.right[level],
                        node_shared,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                        guard,
                    )
                    .is_ok()
                {
                    break;
                }

                node_ref
                    .height_refcount
                    .fetch_sub(1 << HEIGHT_BITS, Ordering::Relaxed);

                search_result = self.search_node(&entry.node.data, guard);
            }
        }

        if node_ref.tower[tower_height - 1]
            .load(Ordering::SeqCst, guard)
            .tag()
            == DELETE_MARK_TAG
        {
            self.search_bound_lower(Bound::Included(&node_ref.data), guard);
        }

        entry.node.ref_count_decrement();
    }

    pub fn range(&self, from: Option<Box<[u8]>>, to: Option<Box<[u8]>>) -> Iter<'_> {
        Iter {
            parent: self,
            from,
            to,
            current: None,
        }
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    fn search_node<'a>(
        &'a self,
        searched: &StructuredMutation,
        guard: &'a Guard,
    ) -> SearchResult<'a> {
        'retry: loop {
            let mut result = SearchResult {
                found_node: None,
                left: [self.head.as_tower(); MAX_HEIGHT],
                right: [Shared::null(); MAX_HEIGHT],
            };

            let mut left: TowerRef<'a> = self.head.as_tower();

            for level in (0..(MAX_HEIGHT - 1)).rev() {
                // Height indexes are 0 based
                let left_next = left[level].load(Ordering::Acquire, guard);

                if left_next.tag() == DELETE_MARK_TAG {
                    continue 'retry;
                }

                let mut current = left_next;
                while let Some(node) = NodeRef::from_shared(current) {
                    let right = node.tower[level].load(Ordering::Acquire, guard);
                    if right.tag() == DELETE_MARK_TAG {
                        if let Some(new_current) = self.unlink(&left[level], node, right, guard) {
                            current = new_current;
                            continue;
                        } else {
                            continue 'retry;
                        }
                    }

                    let cmp_result =
                        MutationComparator::cmp(&self.schema, searched, &node.data).unwrap();
                    match cmp_result {
                        cmp::Ordering::Less => {
                            break;
                        }
                        cmp::Ordering::Equal => {
                            result.found_node = Some(node);
                            break;
                        }
                        _ => {}
                    }

                    left = node.as_tower();
                    current = right;
                }

                result.left[level] = left;
                result.right[level] = current;
            }

            return result;
        }
    }

    fn unlink<'a>(
        &'a self,
        left: &Atomic<Node>,
        current: NodeRef<'a>,
        right: Shared<'a, Node>,
        guard: &Guard,
    ) -> Option<Shared<'a, Node>> {
        match left.compare_exchange(
            Shared::from(current.ptr.as_ptr() as *const Node),
            right.with_tag(0),
            Ordering::Release,
            Ordering::Relaxed,
            guard,
        ) {
            Ok(_) => {
                current.ref_count_decrement();
                Some(right.with_tag(0))
            }
            Err(_) => None,
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

        let mut max_height = self.height.load(Ordering::Relaxed);
        height = usize::min(height, usize::max(max_height + 1, 4));
        while height > max_height {
            match self.height.compare_exchange(
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

    fn next_node<'a>(
        &'a self,
        left: TowerRef<'a>,
        key: &StructuredMutation,
        guard: &'a Guard,
    ) -> Option<NodeRef<'a>> {
        let mut current = left[0].load(Ordering::Acquire, guard);

        if current.tag() == DELETE_MARK_TAG {
            return self.search_bound_lower(Bound::Excluded(key), guard);
        }

        while let Some(node) = NodeRef::from_shared(current) {
            let right = node.tower[0].load(Ordering::Acquire, guard);

            if right.tag() == DELETE_MARK_TAG {
                if let Some(new_current) = self.unlink(&left[0], node, right, guard) {
                    current = new_current;
                    continue;
                } else {
                    return self.search_bound_lower(Bound::Excluded(key), guard);
                }
            }

            return Some(node);
        }

        None
    }

    fn search_bound_lower<'a>(
        &'a self,
        bound: Bound<&StructuredMutation>,
        guard: &'a Guard,
    ) -> Option<NodeRef<'a>> {
        'retry: loop {
            let mut result = None;

            let mut left: TowerRef<'a> = self.head.as_tower();

            for level in (0..(MAX_HEIGHT - 1)).rev() {
                let left_next = left[level].load(Ordering::Acquire, guard);

                if left_next.tag() == DELETE_MARK_TAG {
                    continue 'retry;
                }

                let mut current = left_next;
                while let Some(node) = NodeRef::from_shared(current) {
                    let right = node.tower[level].load(Ordering::Acquire, guard);
                    if right.tag() == DELETE_MARK_TAG {
                        if let Some(new_current) = self.unlink(&left[level], node, right, guard) {
                            current = new_current;
                            continue;
                        } else {
                            continue 'retry;
                        }
                    }

                    let is_above = match bound {
                        Bound::Unbounded => true,
                        Bound::Excluded(key) => {
                            MutationComparator::cmp(&self.schema, &node.data, key)
                                .unwrap()
                                .is_gt()
                        }
                        Bound::Included(key) => {
                            MutationComparator::cmp(&self.schema, &node.data, key)
                                .unwrap()
                                .is_ge()
                        }
                    };
                    if is_above {
                        result = Some(node);
                        break;
                    }

                    left = node.as_tower();
                    current = right;
                }
            }

            return result;
        }
    }

    fn search_bound_lower_pk<'a>(
        &'a self,
        bound: Bound<&[u8]>,
        guard: &'a Guard,
    ) -> Option<NodeRef<'a>> {
        'retry: loop {
            let mut result = None;

            let mut left: TowerRef<'a> = self.head.as_tower();

            for level in (0..(MAX_HEIGHT - 1)).rev() {
                let left_next = left[level].load(Ordering::Acquire, guard);

                if left_next.tag() == DELETE_MARK_TAG {
                    continue 'retry;
                }

                let mut current = left_next;
                while let Some(node) = NodeRef::from_shared(current) {
                    let right = node.tower[level].load(Ordering::Acquire, guard);
                    if right.tag() == DELETE_MARK_TAG {
                        if let Some(new_current) = self.unlink(&left[level], node, right, guard) {
                            current = new_current;
                            continue;
                        } else {
                            continue 'retry;
                        }
                    }

                    let is_above = match bound {
                        Bound::Unbounded => true,
                        Bound::Excluded(key) => {
                            KVPrimaryKeyComparator::cmp(&self.schema, &node.data.primary_key(), key)
                                .unwrap()
                                .is_gt()
                        }
                        Bound::Included(key) => {
                            KVPrimaryKeyComparator::cmp(&self.schema, &node.data.primary_key(), key)
                                .unwrap()
                                .is_ge()
                        }
                    };
                    if is_above {
                        result = Some(node);
                        break;
                    }

                    left = node.as_tower();
                    current = right;
                }
            }

            return result;
        }
    }
}

impl Debug for Skiplist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Skiplist").finish()
    }
}

impl Head {
    fn new() -> Self {
        Self {
            ptrs: Default::default(),
        }
    }

    fn as_tower<'a>(&'a self) -> TowerRef<'a> {
        unsafe { TowerRef::new(self.ptrs.as_ptr() as *mut Tower) }
    }
}

impl Index<usize> for Tower {
    type Output = Atomic<Node>;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*(self.ptrs.as_ptr().add(index)) }
    }
}

impl Node {
    fn alloc(height: usize, ref_count: usize, data: StructuredMutation) -> *const Node {
        let layout = Self::get_layout(height);

        let node = unsafe { alloc(layout).cast::<Node>() };
        if node.is_null() {
            handle_alloc_error(layout);
        }

        unsafe {
            ptr::write(&raw mut (*node).data, data);
            ptr::write(
                &raw mut (*node).height_refcount,
                AtomicUsize::new((ref_count << HEIGHT_BITS) | (height & HEIGHT_MASK)),
            );
            ptr::write_bytes(
                (&raw mut (*node).tower.ptrs) as *mut Atomic<Node>,
                0,
                height,
            );
        }

        node
    }

    fn drop_and_dealloc(ptr: *mut Node) {
        unsafe {
            ptr::drop_in_place(&raw mut (*ptr).data);

            let height = (*ptr).height();
            let layout = Self::get_layout(height);
            dealloc(ptr.cast::<u8>(), layout);
        }
    }

    fn get_layout(height: usize) -> Layout {
        let layout_array = Layout::array::<Atomic<Node>>(height).unwrap();

        let (layout, _) = Layout::new::<Node>().extend(layout_array).unwrap();

        layout.pad_to_align()
    }

    fn height(&self) -> usize {
        self.height_refcount.load(Ordering::Relaxed) & HEIGHT_MASK
    }

    fn mark_tower(&self, guard: &Guard) -> bool {
        let height = self.height();

        for i in (0..height).rev() {
            let tag = self.tower[i]
                .fetch_or(DELETE_MARK_TAG, Ordering::SeqCst, guard)
                .tag();

            if i == 0 && (tag & DELETE_MARK_TAG) > 0 {
                return false;
            }
        }

        true
    }
}

unsafe impl Send for Iter<'_> {}

unsafe impl Sync for Iter<'_> {}

impl<'a> Iterator for Iter<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let guard = unsafe { &*(&pin() as *const _) };

        let mut next = None;
        'retry: loop {
            let next_node = match &self.current {
                Some(entry) => {
                    self.parent
                        .next_node(entry.node.as_tower(), &entry.node.data, guard)
                }
                None => match &self.from {
                    Some(key) => self
                        .parent
                        .search_bound_lower_pk(Bound::Included(key), guard),
                    None => self.parent.search_bound_lower_pk(Bound::Unbounded, guard),
                },
            };

            if let Some(node_ref) = next_node {
                let Some(entry) = Entry::try_acquire(self.parent, node_ref) else {
                    continue 'retry;
                };

                next = Some(entry);
                break;
            } else {
                next = None;
                break;
            }
        }

        self.current = next;

        if let Some(entry) = &self.current {
            if let Some(end_bound) = &self.to {
                if KVPrimaryKeyComparator::cmp(
                    &self.parent.schema,
                    entry.node.data.primary_key(),
                    end_bound,
                )
                .unwrap()
                .is_ge()
                {
                    self.current = None;
                }
            }
        }

        self.current.clone()
    }
}

struct NodeRef<'a> {
    ptr: NonNull<Node>,
    phantom: PhantomData<&'a Node>,
}

impl<'a> NodeRef<'a> {
    fn from_shared(shared: Shared<'a, Node>) -> Option<NodeRef<'a>> {
        match NonNull::new(shared.as_raw() as *mut Node) {
            Some(ptr) => Some(Self {
                ptr,
                phantom: PhantomData,
            }),
            None => None,
        }
    }

    fn ref_count_decrement(&self) {
        let node_ptr = self.ptr.as_ptr();
        let height_refcount = unsafe {
            (*node_ptr)
                .height_refcount
                .fetch_sub(1 << HEIGHT_BITS, Ordering::SeqCst)
        };

        if (height_refcount & !HEIGHT_MASK) == 1 << HEIGHT_BITS {
            atomic::fence(Ordering::Acquire);
            let guard = &pin();
            unsafe {
                guard.defer_unchecked(move || {
                    Node::drop_and_dealloc(node_ptr);
                })
            };
        }
    }

    fn try_increment_ref_count(&self) -> bool {
        let mut height_refcount = self.height_refcount.load(Ordering::Relaxed);

        loop {
            if height_refcount & !HEIGHT_MASK == 0 {
                return false;
            }

            let height_refcount_next = height_refcount.checked_add(1 << HEIGHT_BITS).unwrap();

            match self.height_refcount.compare_exchange_weak(
                height_refcount,
                height_refcount_next,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(current) => height_refcount = current,
            }
        }
    }

    fn as_tower(&self) -> TowerRef<'a> {
        unsafe { TowerRef::new(&raw mut (*self.ptr.as_ptr()).tower) }
    }
}

impl<'a> Clone for NodeRef<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> Copy for NodeRef<'a> {}

impl<'a> Deref for NodeRef<'a> {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

struct TowerRef<'a> {
    ptr: NonNull<Tower>,
    phantom: PhantomData<&'a Tower>,
}

impl<'a> TowerRef<'a> {
    unsafe fn new(ptr: *mut Tower) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            phantom: PhantomData,
        }
    }
}

impl<'a> Clone for TowerRef<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> Copy for TowerRef<'a> {}

impl<'a> Deref for TowerRef<'a> {
    type Target = Tower;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

struct EntryRef<'a> {
    parent: &'a Skiplist,
    node: NodeRef<'a>,
}

impl<'a> EntryRef<'a> {
    pub fn pin(&self) -> Option<Entry<'a>> {
        Entry::try_acquire(self.parent, self.node)
    }

    pub fn as_node(&self) -> NodeRef<'a> {
        self.node
    }
}

impl<'a> Entry<'a> {
    fn try_acquire(parent: &'a Skiplist, node: NodeRef<'a>) -> Option<Entry<'a>> {
        if node.try_increment_ref_count() {
            Some(Entry { parent, node })
        } else {
            None
        }
    }
}

impl Deref for Entry<'_> {
    type Target = StructuredMutation;

    fn deref(&self) -> &Self::Target {
        &self.node.data
    }
}

impl<'a> Clone for Entry<'a> {
    fn clone(&self) -> Self {
        self.node.try_increment_ref_count();

        Self {
            parent: self.parent,
            node: self.node,
        }
    }
}

impl Drop for Entry<'_> {
    fn drop(&mut self) {
        self.node.ref_count_decrement();
    }
}
