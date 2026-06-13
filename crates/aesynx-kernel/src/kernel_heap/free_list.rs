pub const FREE_LIST_EMPTY: usize = 0;

pub(super) fn encode_offset(offset: usize) -> usize {
    offset + 1
}

pub(super) fn decode_offset(encoded: usize) -> usize {
    debug_assert_ne!(encoded, FREE_LIST_EMPTY);
    encoded - 1
}

pub(super) fn read_free_next(ptr: *mut u8) -> usize {
    // SAFETY: Free-list links are stored only in currently free heap blocks.
    // All slab classes are at least pointer-sized and naturally aligned.
    unsafe { (ptr as *const usize).read() }
}

pub(super) fn write_free_next(ptr: *mut u8, next: usize) {
    // SAFETY: Free-list links are written only into currently free heap blocks.
    // All slab classes are at least pointer-sized and naturally aligned.
    unsafe {
        (ptr as *mut usize).write(next);
    }
}

pub(super) fn zero_heap_bytes(ptr: *mut u8, len: usize) {
    // SAFETY: Callers pass validated heap blocks or page runs while holding the
    // allocator metadata lock, so no other live allocation may access this
    // range during allocator-owned zeroing.
    unsafe {
        core::ptr::write_bytes(ptr, 0, len);
    }
}
