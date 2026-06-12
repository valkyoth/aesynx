#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelMappingSmokeStatus {
    pub mapped_pages: u64,
    pub reserved_pages: u64,
    pub text_pages: u64,
    pub rodata_pages: u64,
    pub data_pages: u64,
    pub section_layout_ok: bool,
    pub text_rx_ok: bool,
    pub rodata_read_only_ok: bool,
    pub data_rw_nx_ok: bool,
    pub heap_reserved_ok: bool,
    pub guard_page_ok: bool,
    pub null_page_ok: bool,
    pub hardware_image_ok: bool,
    pub hardware_arena_frames: u64,
    pub hardware_root_allocated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelMappingSmokeError {
    Allocator(aesynx_mm::FrameAllocatorError),
    Mapper(aesynx_mm::PageTableError),
    Plan(aesynx_kernel::kernel_mapping_policy::KernelMappingPlanError),
    AddressOverflow,
    NoUsableWindow,
    UnexpectedPolicy,
}

const TEXT_PHYS: aesynx_abi::PhysAddr = aesynx_abi::PhysAddr::new(0x0030_0000);
const POLICY_TABLES: usize = aesynx_mm::PAGE_TABLE_LEVELS;
const POLICY_MAPPED_FRAMES: usize = 256;
const PAGE_TABLE_ALLOCATOR_WORDS: usize = 2;
const PAGE_TABLE_ALLOCATOR_FRAMES: u64 = 128;
const HEAP_RESERVED_PAGES: u64 = 2;
const GUARD_PAGES: u64 = 1;

pub fn run(
    info: &aesynx_boot::BootInfo<'_>,
    layout: aesynx_kernel::kernel_mapping_policy::KernelSectionLayout,
) -> Result<KernelMappingSmokeStatus, KernelMappingSmokeError> {
    let plan = aesynx_kernel::kernel_mapping_policy::KernelMappingPlan::from_sections(
        layout,
        HEAP_RESERVED_PAGES,
        GUARD_PAGES,
    )
    .map_err(KernelMappingSmokeError::Plan)?;
    let policy = plan.policy();
    let rodata_phys = add_pages_to_phys(TEXT_PHYS, plan.text_pages())?;
    let data_phys = add_pages_to_phys(rodata_phys, plan.rodata_pages())?;

    let mut mapper = aesynx_mm::PageTableMapper::<POLICY_TABLES, POLICY_MAPPED_FRAMES>::new()
        .map_err(KernelMappingSmokeError::Mapper)?;
    mapper
        .map_contiguous(
            policy.text().start(),
            TEXT_PHYS,
            policy.text().pages(),
            aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadExecute),
        )
        .map_err(KernelMappingSmokeError::Mapper)?;
    mapper
        .map_contiguous(
            policy.rodata().start(),
            rodata_phys,
            policy.rodata().pages(),
            aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadOnly),
        )
        .map_err(KernelMappingSmokeError::Mapper)?;
    mapper
        .map_contiguous(
            policy.data().start(),
            data_phys,
            policy.data().pages(),
            aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadWrite),
        )
        .map_err(KernelMappingSmokeError::Mapper)?;

    let report = mapper
        .verify_kernel_mapping_policy(policy)
        .map_err(KernelMappingSmokeError::Mapper)?;

    if report.mapped_pages() != plan.mapped_pages()
        || report.reserved_pages() != plan.reserved_pages()
    {
        return Err(KernelMappingSmokeError::UnexpectedPolicy);
    }
    if !report.text_rx()
        || !report.rodata_read_only()
        || !report.data_rw_nx()
        || !report.reserved_heap_unmapped()
        || !report.guard_page_unmapped()
        || !report.null_page_unmapped()
    {
        return Err(KernelMappingSmokeError::UnexpectedPolicy);
    }
    let arena = allocate_page_table_arena(info)?;
    let root_phys = frame_to_phys(arena.start())?;
    let image = mapper
        .export_x86_64_hardware_image(root_phys)
        .map_err(KernelMappingSmokeError::Mapper)?;
    if image.root_phys() != root_phys
        || image.mapped_pages() != report.mapped_pages()
        || image.used_tables() == 0
        || arena.count() != POLICY_TABLES as u64
    {
        return Err(KernelMappingSmokeError::UnexpectedPolicy);
    }

    Ok(KernelMappingSmokeStatus {
        mapped_pages: report.mapped_pages(),
        reserved_pages: report.reserved_pages(),
        text_pages: plan.text_pages(),
        rodata_pages: plan.rodata_pages(),
        data_pages: plan.data_pages(),
        section_layout_ok: true,
        text_rx_ok: report.text_rx(),
        rodata_read_only_ok: report.rodata_read_only(),
        data_rw_nx_ok: report.data_rw_nx(),
        heap_reserved_ok: report.reserved_heap_unmapped(),
        guard_page_ok: report.guard_page_unmapped(),
        null_page_ok: report.null_page_unmapped(),
        hardware_image_ok: true,
        hardware_arena_frames: arena.count(),
        hardware_root_allocated: true,
    })
}

fn allocate_page_table_arena(
    info: &aesynx_boot::BootInfo<'_>,
) -> Result<aesynx_mm::AllocatedFrames, KernelMappingSmokeError> {
    let (base_frame, frame_count) = first_usable_allocator_window(info)?;
    let mut allocator =
        aesynx_mm::BitmapFrameAllocator::<PAGE_TABLE_ALLOCATOR_WORDS>::new(base_frame, frame_count)
            .map_err(KernelMappingSmokeError::Allocator)?;

    for region in info.memory_map.regions() {
        allocator
            .mark_region(region.start(), region.len, frame_region_kind(region.kind))
            .map_err(KernelMappingSmokeError::Allocator)?;
    }

    allocator
        .allocate_contiguous(POLICY_TABLES as u64)
        .map_err(KernelMappingSmokeError::Allocator)
}

fn first_usable_allocator_window(
    info: &aesynx_boot::BootInfo<'_>,
) -> Result<(aesynx_abi::PhysFrame, u64), KernelMappingSmokeError> {
    for region in info.memory_map.regions() {
        if region.kind != aesynx_boot::MemoryRegionKind::Usable {
            continue;
        }
        let start = align_up_frame(region.start().get())?;
        let end = region
            .end()
            .ok_or(KernelMappingSmokeError::Allocator(
                aesynx_mm::FrameAllocatorError::AddressOverflow,
            ))
            .map(|end| align_down_frame(end.get()))?;
        if end <= start {
            continue;
        }
        let frames = (end - start) / aesynx_mm::FRAME_SIZE;
        if frames >= POLICY_TABLES as u64 {
            return Ok((
                aesynx_abi::PhysFrame::new(start / aesynx_mm::FRAME_SIZE),
                min_u64(frames, PAGE_TABLE_ALLOCATOR_FRAMES),
            ));
        }
    }

    Err(KernelMappingSmokeError::NoUsableWindow)
}

fn add_pages_to_phys(
    phys: aesynx_abi::PhysAddr,
    pages: u64,
) -> Result<aesynx_abi::PhysAddr, KernelMappingSmokeError> {
    let offset = pages
        .checked_mul(aesynx_mm::FRAME_SIZE)
        .ok_or(KernelMappingSmokeError::AddressOverflow)?;
    phys.get()
        .checked_add(offset)
        .map(aesynx_abi::PhysAddr::new)
        .ok_or(KernelMappingSmokeError::AddressOverflow)
}

fn frame_to_phys(
    frame: aesynx_abi::PhysFrame,
) -> Result<aesynx_abi::PhysAddr, KernelMappingSmokeError> {
    frame
        .get()
        .checked_mul(aesynx_mm::FRAME_SIZE)
        .map(aesynx_abi::PhysAddr::new)
        .ok_or(KernelMappingSmokeError::AddressOverflow)
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

fn align_up_frame(value: u64) -> Result<u64, KernelMappingSmokeError> {
    let mask = aesynx_mm::FRAME_SIZE - 1;
    value
        .checked_add(mask)
        .map(|rounded| rounded & !mask)
        .ok_or(KernelMappingSmokeError::Allocator(
            aesynx_mm::FrameAllocatorError::AddressOverflow,
        ))
}

const fn align_down_frame(value: u64) -> u64 {
    value & !(aesynx_mm::FRAME_SIZE - 1)
}

const fn min_u64(left: u64, right: u64) -> u64 {
    if left < right { left } else { right }
}
