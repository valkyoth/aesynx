use alloc::format;

use aesynx_abi::{PhysAddr, PhysFrame};

use super::{BitmapFrameAllocator, FRAME_SIZE, FrameAllocatorError, FrameRegionKind, FrameState};

#[test]
fn allocator_allocates_and_frees_one_frame() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.mark_region(
        PhysAddr::new(FRAME_SIZE),
        8 * FRAME_SIZE,
        FrameRegionKind::Free,
    )?;

    let frame = allocator.allocate_one()?;
    assert_eq!(frame, PhysFrame::new(1));
    assert_eq!(allocator.debug_state(frame), FrameState::Used);
    allocator.free(frame)?;
    assert_eq!(allocator.debug_state(frame), FrameState::Free);

    Ok(())
}

#[test]
fn allocator_detects_double_free() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.mark_region(
        PhysAddr::new(FRAME_SIZE),
        8 * FRAME_SIZE,
        FrameRegionKind::Free,
    )?;

    let frame = allocator.allocate_one()?;
    allocator.free(frame)?;

    assert_eq!(allocator.free(frame), Err(FrameAllocatorError::DoubleFree));
    Ok(())
}

#[test]
fn allocator_allocates_contiguous_frames() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.mark_region(
        PhysAddr::new(FRAME_SIZE),
        FRAME_SIZE,
        FrameRegionKind::Kernel,
    )?;
    allocator.mark_region(
        PhysAddr::new(2 * FRAME_SIZE),
        7 * FRAME_SIZE,
        FrameRegionKind::Free,
    )?;

    let frames = allocator.allocate_contiguous(3)?;

    assert_eq!(frames.start(), PhysFrame::new(2));
    assert_eq!(frames.count(), 3);
    assert_eq!(allocator.debug_state(PhysFrame::new(2)), FrameState::Used);
    assert_eq!(allocator.debug_state(PhysFrame::new(4)), FrameState::Used);
    assert_eq!(allocator.debug_state(PhysFrame::new(5)), FrameState::Free);
    Ok(())
}

#[test]
fn allocated_frames_debug_redacts_start_frame() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.mark_region(
        PhysAddr::new(FRAME_SIZE),
        8 * FRAME_SIZE,
        FrameRegionKind::Free,
    )?;

    let frames = allocator.allocate_contiguous(3)?;
    let debug = format!("{frames:?}");

    assert!(debug.contains("AllocatedFrames"));
    assert!(debug.contains("start: \"<redacted>\""));
    assert!(debug.contains("count: 3"));
    assert!(!debug.contains("PhysFrame"));
    Ok(())
}

#[test]
fn allocator_rejects_overlapping_regions() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.mark_region(
        PhysAddr::new(FRAME_SIZE),
        4 * FRAME_SIZE,
        FrameRegionKind::Free,
    )?;

    assert_eq!(
        allocator.mark_region(
            PhysAddr::new(3 * FRAME_SIZE),
            2 * FRAME_SIZE,
            FrameRegionKind::Kernel,
        ),
        Err(FrameAllocatorError::RegionOverlap)
    );
    Ok(())
}

#[test]
fn mark_region_overlap_is_atomic() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.mark_region(
        PhysAddr::new(3 * FRAME_SIZE),
        FRAME_SIZE,
        FrameRegionKind::Free,
    )?;

    let before = allocator;

    assert_eq!(
        allocator.mark_region(
            PhysAddr::new(FRAME_SIZE),
            3 * FRAME_SIZE,
            FrameRegionKind::Kernel,
        ),
        Err(FrameAllocatorError::RegionOverlap)
    );
    assert_eq!(allocator, before);
    assert_eq!(
        allocator.debug_state(PhysFrame::new(1)),
        FrameState::Unknown
    );
    assert_eq!(
        allocator.debug_state(PhysFrame::new(2)),
        FrameState::Unknown
    );
    assert_eq!(allocator.debug_state(PhysFrame::new(3)), FrameState::Free);
    Ok(())
}

#[test]
fn allocator_reports_reserved_states() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.mark_region(
        PhysAddr::new(FRAME_SIZE),
        FRAME_SIZE,
        FrameRegionKind::Kernel,
    )?;
    allocator.mark_region(
        PhysAddr::new(2 * FRAME_SIZE),
        FRAME_SIZE,
        FrameRegionKind::Bootloader,
    )?;
    allocator.mark_region(
        PhysAddr::new(3 * FRAME_SIZE),
        FRAME_SIZE,
        FrameRegionKind::Device,
    )?;
    allocator.mark_region(
        PhysAddr::new(4 * FRAME_SIZE),
        FRAME_SIZE,
        FrameRegionKind::Acpi,
    )?;
    allocator.mark_region(
        PhysAddr::new(5 * FRAME_SIZE),
        FRAME_SIZE,
        FrameRegionKind::Bad,
    )?;

    assert_eq!(allocator.debug_state(PhysFrame::new(1)), FrameState::Kernel);
    assert_eq!(
        allocator.debug_state(PhysFrame::new(2)),
        FrameState::Bootloader
    );
    assert_eq!(allocator.debug_state(PhysFrame::new(3)), FrameState::Device);
    assert_eq!(allocator.debug_state(PhysFrame::new(4)), FrameState::Acpi);
    assert_eq!(allocator.debug_state(PhysFrame::new(5)), FrameState::Bad);
    assert_eq!(
        allocator.debug_state(PhysFrame::new(6)),
        FrameState::Unknown
    );
    Ok(())
}

#[test]
fn free_contiguous_double_free_is_atomic() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.mark_region(
        PhysAddr::new(FRAME_SIZE),
        8 * FRAME_SIZE,
        FrameRegionKind::Free,
    )?;
    let frames = allocator.allocate_contiguous(3)?;
    allocator.free(PhysFrame::new(2))?;

    let before = allocator;

    assert_eq!(
        allocator.free_contiguous(frames),
        Err(FrameAllocatorError::DoubleFree)
    );
    assert_eq!(allocator, before);
    assert_eq!(allocator.debug_state(PhysFrame::new(1)), FrameState::Used);
    assert_eq!(allocator.debug_state(PhysFrame::new(2)), FrameState::Free);
    assert_eq!(allocator.debug_state(PhysFrame::new(3)), FrameState::Used);
    Ok(())
}

#[test]
fn allocator_status_counts_state_classes() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.mark_region(
        PhysAddr::new(FRAME_SIZE),
        4 * FRAME_SIZE,
        FrameRegionKind::Free,
    )?;
    allocator.mark_region(
        PhysAddr::new(5 * FRAME_SIZE),
        2 * FRAME_SIZE,
        FrameRegionKind::Reserved,
    )?;
    let _frame = allocator.allocate_one()?;
    let status = allocator.status();

    assert_eq!(status.total_frames(), 8);
    assert_eq!(status.known_frames(), 6);
    assert_eq!(status.free_frames(), 3);
    assert_eq!(status.used_frames(), 1);
    assert_eq!(status.reserved_frames(), 2);
    assert_eq!(status.unknown_frames(), 2);
    assert_eq!(allocator.status_checked(), Ok(status));
    Ok(())
}

#[test]
fn allocator_checked_status_rejects_unknown_classification() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.free.set(0, true)?;

    assert_eq!(
        allocator.status_checked(),
        Err(FrameAllocatorError::CorruptAllocator)
    );
    assert_eq!(allocator.status().known_frames(), 0);
    assert_eq!(allocator.status().free_frames(), 1);
    assert_eq!(allocator.status().used_frames(), 0);
    Ok(())
}

#[test]
fn allocator_checked_status_rejects_overlapping_classes() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.known.set(0, true)?;
    allocator.free.set(0, true)?;
    allocator.kernel.set(0, true)?;

    assert_eq!(
        allocator.status_checked(),
        Err(FrameAllocatorError::CorruptAllocator)
    );
    assert_eq!(allocator.status().known_frames(), 1);
    assert_eq!(allocator.status().free_frames(), 1);
    assert_eq!(allocator.status().reserved_frames(), 1);
    assert_eq!(allocator.status().used_frames(), 0);
    Ok(())
}

#[test]
fn allocator_rejects_reserved_free() -> Result<(), FrameAllocatorError> {
    let mut allocator = BitmapFrameAllocator::<1>::new(PhysFrame::new(1), 8)?;
    allocator.mark_region(
        PhysAddr::new(FRAME_SIZE),
        FRAME_SIZE,
        FrameRegionKind::Kernel,
    )?;

    assert_eq!(
        allocator.free(PhysFrame::new(1)),
        Err(FrameAllocatorError::FrameNotAllocatable)
    );
    Ok(())
}
