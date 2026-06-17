pub const FREE_LIST_EMPTY: usize = 0;

pub(super) fn encode_offset(offset: usize) -> usize {
    offset + 1
}

pub(super) const fn decode_offset(encoded: usize) -> Option<usize> {
    encoded.checked_sub(1)
}

pub(super) const fn valid_free_offset(
    offset: usize,
    total_bytes: usize,
    block_size: usize,
) -> bool {
    offset < total_bytes && offset.is_multiple_of(block_size)
}

pub(super) const fn decode_valid_offset(
    encoded: usize,
    total_bytes: usize,
    block_size: usize,
) -> Option<usize> {
    let offset = match decode_offset(encoded) {
        Some(offset) => offset,
        None => return None,
    };
    if valid_free_offset(offset, total_bytes, block_size) {
        Some(offset)
    } else {
        None
    }
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

#[cfg(test)]
mod tests {
    use super::{FREE_LIST_EMPTY, decode_offset, encode_offset};

    #[test]
    fn free_list_offset_decode_rejects_empty_sentinel() {
        assert_eq!(decode_offset(FREE_LIST_EMPTY), None);
        assert_eq!(decode_offset(encode_offset(4096)), Some(4096));
    }
}
