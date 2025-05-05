use std::{
    ffi::c_void,
    fs::{File, OpenOptions},
    os::fd::AsFd,
    ptr,
    slice::{self, from_raw_parts},
    sync::atomic::Ordering,
};

use rustix::{
    fs::ftruncate,
    mm::{MapFlags, MsyncFlags, ProtFlags, mmap, msync, munmap},
};

use crate::segment::{Segment, SegmentId};

pub(crate) fn read_segment<'a>() -> Segment<'a> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("/tmp/wal.test")
        .unwrap();
    let fd = file.as_fd();

    ftruncate(fd, Segment::SEGMENT_LENGTH).unwrap();
    let mapped = unsafe {
        mmap(
            ptr::null_mut::<c_void>(),
            Segment::SEGMENT_LENGTH as usize,
            ProtFlags::READ | ProtFlags::WRITE,
            MapFlags::SHARED,
            fd,
            0,
        )
    }
    .unwrap();

    let data =
        unsafe { slice::from_raw_parts_mut(mapped.cast::<u8>(), Segment::SEGMENT_LENGTH as usize) };

    todo!()
}

pub(crate) fn sync_segment(segment: &mut Segment) {
    let synced_ptr = segment.synced_ptr;
    let tail_ptr = segment.tail_ptr;

    let addr = segment.data.as_mut_ptr().cast::<c_void>();
    let len = tail_ptr - synced_ptr;

    unsafe { msync(addr, len as usize, MsyncFlags::SYNC).unwrap() }

    segment.synced_ptr = tail_ptr;
}

pub(crate) fn drop_segment(segment: &mut Segment) {
    let ptr = segment.data.as_mut_ptr().cast::<c_void>();
    unsafe {
        munmap(ptr, Segment::SEGMENT_LENGTH as usize).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::read_segment;

    #[test]
    fn test() {
        let segment = read_segment();
        drop(segment);
    }
}
