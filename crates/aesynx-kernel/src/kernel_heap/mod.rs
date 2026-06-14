#![cfg_attr(
    any(
        test,
        feature = "panic-smoke",
        feature = "exception-smoke",
        feature = "timer-smoke"
    ),
    allow(dead_code)
)]

mod allocator;
mod backing;
mod free_list;
mod global_alloc;
mod layout;
mod lock;
mod stats;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::alloc::Layout;

pub use allocator::KernelHeapAllocator;
pub use layout::{KERNEL_HEAP_BYTES, KERNEL_HEAP_PAGE_SIZE, SLAB_CLASS_COUNT};
pub use stats::{KernelHeapError, KernelHeapStatus};

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub fn smoke(allocator: &KernelHeapAllocator) -> Result<KernelHeapStatus, KernelHeapError> {
    allocator.init()?;

    let boxed = Box::new(0x0a18_0001_u64);
    let box_ok = *boxed == 0x0a18_0001_u64;

    let mut values = Vec::new();
    values
        .try_reserve_exact(4)
        .map_err(|_error| KernelHeapError::SmokeFailed)?;
    values.push(3_u64);
    values.push(5_u64);
    values.push(8_u64);
    values.push(13_u64);
    let vec_ok = values.as_slice() == [3, 5, 8, 13];

    let mut map = BTreeMap::new();
    map.insert(2_u8, 3_u8);
    map.insert(5_u8, 8_u8);
    let btree_ok = map.get(&5).copied() == Some(8) && map.len() == 2;

    let slab_reuse_ok = slab_reuse_smoke(allocator)?;
    let page_run_ok = page_run_smoke(allocator)?;
    let stress_ok = stress_smoke(allocator)?;
    let double_free_detected = double_free_smoke(allocator)?;
    let invalid_free_detected = invalid_free_smoke(allocator)?;

    let mut oversized = Vec::<u8>::new();
    let oom_rejected = oversized.try_reserve_exact(KERNEL_HEAP_BYTES * 2).is_err();

    if !(box_ok
        && vec_ok
        && btree_ok
        && slab_reuse_ok
        && page_run_ok
        && stress_ok
        && double_free_detected
        && invalid_free_detected
        && oom_rejected)
    {
        return Err(KernelHeapError::SmokeFailed);
    }

    let stats = allocator.stats()?;
    Ok(KernelHeapStatus {
        heap_bytes: stats.heap_bytes,
        allocated_bytes: stats.allocated_bytes,
        peak_allocated_bytes: stats.peak_allocated_bytes,
        slab_classes: SLAB_CLASS_COUNT,
        slab_allocations: stats.slab_allocations,
        page_allocations: stats.page_allocations,
        frees: stats.frees,
        double_free_detected: stats.double_free_detected,
        invalid_free_detected: stats.invalid_free_detected,
        corrupt_free_list_detected: stats.corrupt_free_list_detected,
        box_ok,
        vec_ok,
        btree_ok,
        slab_reuse_ok,
        page_run_ok,
        stress_ok,
        oom_rejected,
    })
}

fn slab_reuse_smoke(allocator: &KernelHeapAllocator) -> Result<bool, KernelHeapError> {
    let layout =
        Layout::from_size_align(64, 64).map_err(|_error| KernelHeapError::InvalidLayout)?;
    let first = allocator.allocate_checked(layout)?;
    allocator.deallocate_checked(first, layout)?;
    let second = allocator.allocate_checked(layout)?;
    let reused = first == second;
    allocator.deallocate_checked(second, layout)?;
    Ok(reused)
}

fn page_run_smoke(allocator: &KernelHeapAllocator) -> Result<bool, KernelHeapError> {
    let layout = Layout::from_size_align((KERNEL_HEAP_PAGE_SIZE * 3) + 17, KERNEL_HEAP_PAGE_SIZE)
        .map_err(|_error| KernelHeapError::InvalidLayout)?;
    let ptr = allocator.allocate_checked(layout)?;
    let page_aligned = (ptr as usize) & (KERNEL_HEAP_PAGE_SIZE - 1) == 0;
    allocator.deallocate_checked(ptr, layout)?;
    Ok(page_aligned)
}

fn stress_smoke(allocator: &KernelHeapAllocator) -> Result<bool, KernelHeapError> {
    let layouts = [
        Layout::from_size_align(16, 8).map_err(|_error| KernelHeapError::InvalidLayout)?,
        Layout::from_size_align(31, 16).map_err(|_error| KernelHeapError::InvalidLayout)?,
        Layout::from_size_align(128, 64).map_err(|_error| KernelHeapError::InvalidLayout)?,
        Layout::from_size_align(900, 128).map_err(|_error| KernelHeapError::InvalidLayout)?,
        Layout::from_size_align(3000, KERNEL_HEAP_PAGE_SIZE)
            .map_err(|_error| KernelHeapError::InvalidLayout)?,
    ];
    let mut ptrs = [core::ptr::null_mut(); 5];
    let mut index = 0usize;
    while index < layouts.len() {
        ptrs[index] = allocator.allocate_checked(layouts[index])?;
        index += 1;
    }
    while index > 0 {
        index -= 1;
        allocator.deallocate_checked(ptrs[index], layouts[index])?;
    }
    Ok(true)
}

fn double_free_smoke(allocator: &KernelHeapAllocator) -> Result<bool, KernelHeapError> {
    let layout =
        Layout::from_size_align(32, 16).map_err(|_error| KernelHeapError::InvalidLayout)?;
    let ptr = allocator.allocate_checked(layout)?;
    allocator.deallocate_checked(ptr, layout)?;
    Ok(allocator.deallocate_checked(ptr, layout) == Err(KernelHeapError::DoubleFree))
}

fn invalid_free_smoke(allocator: &KernelHeapAllocator) -> Result<bool, KernelHeapError> {
    let layout =
        Layout::from_size_align(128, 64).map_err(|_error| KernelHeapError::InvalidLayout)?;
    let wrong = Layout::from_size_align(64, 64).map_err(|_error| KernelHeapError::InvalidLayout)?;
    let ptr = allocator.allocate_checked(layout)?;
    let detected = allocator.deallocate_checked(ptr, wrong) == Err(KernelHeapError::InvalidFree);
    allocator.deallocate_checked(ptr, layout)?;
    Ok(detected)
}
