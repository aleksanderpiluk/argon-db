use std::collections::BTreeSet;

struct StorageSharedGlobalState {
    keyspaces: BTreeSet<StorageSharedKeyspaceState>,
}

struct StorageSharedKeyspaceState {}
