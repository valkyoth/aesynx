const EARLY_FRAME_ALLOCATOR_WORDS: usize = 2;
const EARLY_FRAME_ALLOCATOR_FRAMES: u64 = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameAllocatorSmokeStatus {
    pub total_frames: u64,
    pub known_frames: u64,
    pub free_before: u64,
    pub free_after: u64,
    pub reserved_frames: u64,
    pub contiguous_count: u64,
    pub double_free_check: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameAllocatorSmokeError {
    Allocator(aesynx_mm::FrameAllocatorError),
    NoUsableWindow,
    StateMismatch,
    DoubleFreeCheckFailed,
}

pub fn run(
    info: &aesynx_boot::BootInfo<'_>,
) -> Result<FrameAllocatorSmokeStatus, FrameAllocatorSmokeError> {
    let (base_frame, frame_count) = first_usable_allocator_window(info)?;
    let mut allocator = aesynx_mm::BitmapFrameAllocator::<EARLY_FRAME_ALLOCATOR_WORDS>::new(
        base_frame,
        frame_count,
    )
    .map_err(FrameAllocatorSmokeError::Allocator)?;

    for region in info.memory_map.regions() {
        allocator
            .mark_region(region.start(), region.len, frame_region_kind(region.kind))
            .map_err(FrameAllocatorSmokeError::Allocator)?;
    }

    let before = allocator
        .status_checked()
        .map_err(FrameAllocatorSmokeError::Allocator)?;
    let contiguous = allocator
        .allocate_contiguous(2)
        .map_err(FrameAllocatorSmokeError::Allocator)?;
    let single = allocator
        .allocate_one()
        .map_err(FrameAllocatorSmokeError::Allocator)?;
    if allocator.debug_state(single) != aesynx_mm::FrameState::Used {
        return Err(FrameAllocatorSmokeError::StateMismatch);
    }
    allocator
        .free(single)
        .map_err(FrameAllocatorSmokeError::Allocator)?;
    if allocator.free(single) != Err(aesynx_mm::FrameAllocatorError::DoubleFree) {
        return Err(FrameAllocatorSmokeError::DoubleFreeCheckFailed);
    }
    allocator
        .free_contiguous(contiguous)
        .map_err(FrameAllocatorSmokeError::Allocator)?;
    let after = allocator
        .status_checked()
        .map_err(FrameAllocatorSmokeError::Allocator)?;
    if before.free_frames() != after.free_frames() {
        return Err(FrameAllocatorSmokeError::StateMismatch);
    }

    Ok(FrameAllocatorSmokeStatus {
        total_frames: after.total_frames(),
        known_frames: after.known_frames(),
        free_before: before.free_frames(),
        free_after: after.free_frames(),
        reserved_frames: after.reserved_frames(),
        contiguous_count: contiguous.count(),
        double_free_check: true,
    })
}

fn first_usable_allocator_window(
    info: &aesynx_boot::BootInfo<'_>,
) -> Result<(aesynx_abi::PhysFrame, u64), FrameAllocatorSmokeError> {
    for region in info.memory_map.regions() {
        if region.kind != aesynx_boot::MemoryRegionKind::Usable {
            continue;
        }
        let start =
            align_up_frame(region.start().get()).map_err(FrameAllocatorSmokeError::Allocator)?;
        let end = region
            .end()
            .ok_or(FrameAllocatorSmokeError::Allocator(
                aesynx_mm::FrameAllocatorError::AddressOverflow,
            ))
            .map(|end| align_down_frame(end.get()))?;
        if end <= start {
            continue;
        }
        let frames = (end - start) / aesynx_mm::FRAME_SIZE;
        if frames >= 3 {
            return Ok((
                aesynx_abi::PhysFrame::new(start / aesynx_mm::FRAME_SIZE),
                min_u64(frames, EARLY_FRAME_ALLOCATOR_FRAMES),
            ));
        }
    }

    Err(FrameAllocatorSmokeError::NoUsableWindow)
}

fn frame_region_kind(kind: aesynx_boot::MemoryRegionKind) -> aesynx_mm::FrameRegionKind {
    match kind {
        aesynx_boot::MemoryRegionKind::Usable => aesynx_mm::FrameRegionKind::Free,
        aesynx_boot::MemoryRegionKind::Reserved => aesynx_mm::FrameRegionKind::Reserved,
        aesynx_boot::MemoryRegionKind::Kernel => aesynx_mm::FrameRegionKind::Kernel,
        aesynx_boot::MemoryRegionKind::Bootloader => aesynx_mm::FrameRegionKind::Bootloader,
        aesynx_boot::MemoryRegionKind::Framebuffer => aesynx_mm::FrameRegionKind::Device,
        aesynx_boot::MemoryRegionKind::Acpi => aesynx_mm::FrameRegionKind::Acpi,
        aesynx_boot::MemoryRegionKind::Bad => aesynx_mm::FrameRegionKind::Bad,
    }
}

fn align_up_frame(value: u64) -> Result<u64, aesynx_mm::FrameAllocatorError> {
    let mask = aesynx_mm::FRAME_SIZE - 1;
    value
        .checked_add(mask)
        .map(|rounded| rounded & !mask)
        .ok_or(aesynx_mm::FrameAllocatorError::AddressOverflow)
}

const fn align_down_frame(value: u64) -> u64 {
    value & !(aesynx_mm::FRAME_SIZE - 1)
}

const fn min_u64(left: u64, right: u64) -> u64 {
    if left < right { left } else { right }
}
