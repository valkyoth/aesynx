use core::alloc::Layout;

use super::test_support;
use super::{KERNEL_HEAP_PAGE_SIZE, KernelHeapAllocator, KernelHeapError};

#[repr(align(4096))]
struct TestHeap([u8; KERNEL_HEAP_PAGE_SIZE * 8]);

fn init_test_allocator() -> Result<(KernelHeapAllocator, TestHeap), KernelHeapError> {
    let allocator = KernelHeapAllocator::new();
    let heap = TestHeap([0; KERNEL_HEAP_PAGE_SIZE * 8]);
    allocator.init_with_bounds(heap.0.as_ptr() as usize, heap.0.len())?;
    Ok((allocator, heap))
}

#[test]
fn allocation_rejects_before_initialization() -> Result<(), KernelHeapError> {
    let allocator = KernelHeapAllocator::new();
    let layout = Layout::from_size_align(8, 8).map_err(|_error| KernelHeapError::InvalidLayout)?;

    assert_eq!(
        allocator.allocate_checked(layout),
        Err(KernelHeapError::NotInitialized)
    );
    assert_eq!(
        allocator.allocated_bytes(),
        Err(KernelHeapError::NotInitialized)
    );
    Ok(())
}

#[test]
fn slab_allocations_are_reused_after_free() -> Result<(), KernelHeapError> {
    let (allocator, _heap) = init_test_allocator()?;
    let layout =
        Layout::from_size_align(64, 64).map_err(|_error| KernelHeapError::InvalidLayout)?;

    let first = allocator.allocate_checked(layout)?;
    assert_eq!((first as usize) & 63, 0);
    allocator.deallocate_checked(first, layout)?;
    let second = allocator.allocate_checked(layout)?;

    assert_eq!(first, second);
    allocator.deallocate_checked(second, layout)?;
    assert_eq!(allocator.allocated_bytes()?, 0);
    Ok(())
}

#[test]
fn large_allocations_use_page_runs_and_free_them() -> Result<(), KernelHeapError> {
    let (allocator, _heap) = init_test_allocator()?;
    let layout = Layout::from_size_align((KERNEL_HEAP_PAGE_SIZE * 2) + 1, KERNEL_HEAP_PAGE_SIZE)
        .map_err(|_error| KernelHeapError::InvalidLayout)?;

    let first = allocator.allocate_checked(layout)?;
    assert_eq!((first as usize) & (KERNEL_HEAP_PAGE_SIZE - 1), 0);
    allocator.deallocate_checked(first, layout)?;
    let second = allocator.allocate_checked(layout)?;

    assert_eq!(first, second);
    allocator.deallocate_checked(second, layout)?;
    assert_eq!(allocator.allocated_bytes()?, 0);
    Ok(())
}

#[test]
fn large_free_rejects_wrong_layout_without_releasing_run() -> Result<(), KernelHeapError> {
    let (allocator, _heap) = init_test_allocator()?;
    let layout = Layout::from_size_align((KERNEL_HEAP_PAGE_SIZE * 2) + 1, KERNEL_HEAP_PAGE_SIZE)
        .map_err(|_error| KernelHeapError::InvalidLayout)?;
    let wrong = Layout::from_size_align(KERNEL_HEAP_PAGE_SIZE, KERNEL_HEAP_PAGE_SIZE)
        .map_err(|_error| KernelHeapError::InvalidLayout)?;

    let ptr = allocator.allocate_checked(layout)?;
    assert_eq!(
        allocator.deallocate_checked(ptr, wrong),
        Err(KernelHeapError::InvalidFree)
    );
    assert!(allocator.stats()?.invalid_free_detected);
    assert_ne!(allocator.allocated_bytes()?, 0);
    allocator.deallocate_checked(ptr, layout)?;
    assert_eq!(allocator.allocated_bytes()?, 0);
    Ok(())
}

#[test]
fn invalid_slab_free_is_reported_without_releasing_block() -> Result<(), KernelHeapError> {
    let (allocator, _heap) = init_test_allocator()?;
    let layout =
        Layout::from_size_align(128, 64).map_err(|_error| KernelHeapError::InvalidLayout)?;
    let wrong = Layout::from_size_align(64, 64).map_err(|_error| KernelHeapError::InvalidLayout)?;

    let ptr = allocator.allocate_checked(layout)?;
    assert_eq!(
        allocator.deallocate_checked(ptr, wrong),
        Err(KernelHeapError::InvalidFree)
    );
    assert!(allocator.stats()?.invalid_free_detected);
    assert_ne!(allocator.allocated_bytes()?, 0);

    allocator.deallocate_checked(ptr, layout)?;
    assert_eq!(allocator.allocated_bytes()?, 0);
    Ok(())
}

#[test]
fn double_free_is_reported_without_reallocating_block() -> Result<(), KernelHeapError> {
    let (allocator, _heap) = init_test_allocator()?;
    let layout =
        Layout::from_size_align(32, 16).map_err(|_error| KernelHeapError::InvalidLayout)?;

    let ptr = allocator.allocate_checked(layout)?;
    allocator.deallocate_checked(ptr, layout)?;
    assert_eq!(
        allocator.deallocate_checked(ptr, layout),
        Err(KernelHeapError::DoubleFree)
    );
    assert!(allocator.stats()?.double_free_detected);
    Ok(())
}

#[test]
fn corrupt_free_list_head_is_rejected_before_pointer_deref() -> Result<(), KernelHeapError> {
    let (allocator, _heap) = init_test_allocator()?;
    let layout =
        Layout::from_size_align(64, 64).map_err(|_error| KernelHeapError::InvalidLayout)?;
    let ptr = allocator.allocate_checked(layout)?;
    let out_of_heap_offset = (KERNEL_HEAP_PAGE_SIZE * 8) + 64;

    allocator.corrupt_free_head_for_test(layout, out_of_heap_offset);
    assert_eq!(
        allocator.deallocate_checked(ptr, layout),
        Err(KernelHeapError::CorruptFreeList)
    );
    assert!(allocator.stats()?.corrupt_free_list_detected);
    assert_ne!(allocator.allocated_bytes()?, 0);
    Ok(())
}

#[test]
fn slab_memory_is_zeroed_before_reuse() -> Result<(), KernelHeapError> {
    let (allocator, _heap) = init_test_allocator()?;
    let layout =
        Layout::from_size_align(64, 64).map_err(|_error| KernelHeapError::InvalidLayout)?;

    let first = allocator.allocate_checked(layout)?;
    test_support::fill_allocation(first, layout.size(), 0xa5)?;
    allocator.deallocate_checked(first, layout)?;
    let second = allocator.allocate_checked(layout)?;
    assert_eq!(first, second);

    assert!(test_support::allocation_is_zero(second, layout.size())?);

    allocator.deallocate_checked(second, layout)?;
    Ok(())
}

#[test]
fn page_run_memory_is_zeroed_before_reuse() -> Result<(), KernelHeapError> {
    let (allocator, _heap) = init_test_allocator()?;
    let layout = Layout::from_size_align(KERNEL_HEAP_PAGE_SIZE + 17, KERNEL_HEAP_PAGE_SIZE)
        .map_err(|_error| KernelHeapError::InvalidLayout)?;
    let run_bytes = KERNEL_HEAP_PAGE_SIZE * 2;

    let first = allocator.allocate_checked(layout)?;
    test_support::fill_allocation(first, run_bytes, 0xa5)?;
    allocator.deallocate_checked(first, layout)?;
    let second = allocator.allocate_checked(layout)?;
    assert_eq!(first, second);

    assert!(test_support::allocation_is_zero(second, run_bytes)?);

    allocator.deallocate_checked(second, layout)?;
    Ok(())
}

#[test]
fn page_run_memory_is_zeroed_on_first_allocation() -> Result<(), KernelHeapError> {
    let allocator = KernelHeapAllocator::new();
    let heap = TestHeap([0xa5; KERNEL_HEAP_PAGE_SIZE * 8]);
    allocator.init_with_bounds(heap.0.as_ptr() as usize, heap.0.len())?;
    let layout = Layout::from_size_align(KERNEL_HEAP_PAGE_SIZE + 17, KERNEL_HEAP_PAGE_SIZE)
        .map_err(|_error| KernelHeapError::InvalidLayout)?;
    let run_bytes = KERNEL_HEAP_PAGE_SIZE * 2;

    let ptr = allocator.allocate_checked(layout)?;
    assert!(test_support::allocation_is_zero(ptr, run_bytes)?);

    allocator.deallocate_checked(ptr, layout)?;
    Ok(())
}

#[test]
fn stats_track_allocations_frees_and_peak() -> Result<(), KernelHeapError> {
    let (allocator, _heap) = init_test_allocator()?;
    let slab = Layout::from_size_align(128, 32).map_err(|_error| KernelHeapError::InvalidLayout)?;
    let large = Layout::from_size_align(KERNEL_HEAP_PAGE_SIZE + 1, KERNEL_HEAP_PAGE_SIZE)
        .map_err(|_error| KernelHeapError::InvalidLayout)?;

    let slab_ptr = allocator.allocate_checked(slab)?;
    let large_ptr = allocator.allocate_checked(large)?;
    let peak = allocator.stats()?.peak_allocated_bytes;
    allocator.deallocate_checked(large_ptr, large)?;
    allocator.deallocate_checked(slab_ptr, slab)?;
    let stats = allocator.stats()?;

    assert_eq!(stats.allocated_bytes, 0);
    assert!(peak >= KERNEL_HEAP_PAGE_SIZE * 2);
    assert_eq!(stats.slab_allocations, 1);
    assert_eq!(stats.page_allocations, 1);
    assert_eq!(stats.frees, 2);
    assert!(!stats.corrupt_free_list_detected);
    Ok(())
}

#[test]
fn oversized_allocation_fails_without_advancing_stats() -> Result<(), KernelHeapError> {
    let (allocator, _heap) = init_test_allocator()?;
    let layout = Layout::from_size_align(KERNEL_HEAP_PAGE_SIZE * 16, KERNEL_HEAP_PAGE_SIZE)
        .map_err(|_error| KernelHeapError::InvalidLayout)?;
    let before = allocator.stats()?;

    assert_eq!(
        allocator.allocate_checked(layout),
        Err(KernelHeapError::OutOfMemory)
    );
    let after = allocator.stats()?;
    assert_eq!(before.allocated_bytes, after.allocated_bytes);
    assert_eq!(before.peak_allocated_bytes, after.peak_allocated_bytes);
    Ok(())
}

#[test]
fn allocator_init_is_one_shot() -> Result<(), KernelHeapError> {
    let allocator = KernelHeapAllocator::new();
    let heap = TestHeap([0; KERNEL_HEAP_PAGE_SIZE * 8]);

    allocator.init_with_bounds(heap.0.as_ptr() as usize, heap.0.len())?;
    assert_eq!(
        allocator.init_with_bounds(heap.0.as_ptr() as usize, heap.0.len()),
        Err(KernelHeapError::AlreadyInitialized)
    );
    Ok(())
}
