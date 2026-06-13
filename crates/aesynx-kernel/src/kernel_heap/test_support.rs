use super::stats::KernelHeapError;

pub(super) fn fill_allocation(ptr: *mut u8, len: usize, byte: u8) -> Result<(), KernelHeapError> {
    if ptr.is_null() {
        return Err(KernelHeapError::InvalidFree);
    }
    // SAFETY: Test callers pass live allocations returned by the heap and a
    // length bounded by the allocation's rounded size.
    unsafe {
        core::ptr::write_bytes(ptr, byte, len);
    }
    Ok(())
}

pub(super) fn allocation_is_zero(ptr: *mut u8, len: usize) -> Result<bool, KernelHeapError> {
    if ptr.is_null() {
        return Err(KernelHeapError::InvalidFree);
    }
    let mut cursor = 0usize;
    while cursor < len {
        // SAFETY: Test callers pass live allocations returned by the heap and a
        // length bounded by the allocation's rounded size.
        let byte = unsafe { core::ptr::read(ptr.add(cursor)) };
        if byte != 0 {
            return Ok(false);
        }
        cursor += 1;
    }
    Ok(true)
}
