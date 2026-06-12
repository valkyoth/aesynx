#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelMappingSmokeStatus {
    pub mapped_pages: u64,
    pub reserved_pages: u64,
    pub text_pages: u64,
    pub rodata_pages: u64,
    pub data_pages: u64,
    pub text_rx_ok: bool,
    pub rodata_read_only_ok: bool,
    pub data_rw_nx_ok: bool,
    pub heap_reserved_ok: bool,
    pub guard_page_ok: bool,
    pub null_page_ok: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelMappingSmokeError {
    Mapper(aesynx_mm::PageTableError),
    AddressOverflow,
    InvalidSectionLayout,
    UnexpectedPolicy,
}

const TEXT_PHYS: aesynx_abi::PhysAddr = aesynx_abi::PhysAddr::new(0x0030_0000);
const POLICY_TABLES: usize = aesynx_mm::PAGE_TABLE_LEVELS;
const POLICY_MAPPED_FRAMES: usize = 256;
const HEAP_RESERVED_PAGES: u64 = 2;
const GUARD_PAGES: u64 = 1;

pub fn run(
    layout: crate::kernel_sections::KernelSectionLayout,
) -> Result<KernelMappingSmokeStatus, KernelMappingSmokeError> {
    let text_pages = section_pages(layout.text_start, layout.text_end)?;
    let rodata_pages = section_pages(layout.rodata_start, layout.rodata_end)?;
    let data_pages = section_pages(layout.data_start, layout.data_end)?;
    ensure_ordered_layout(layout)?;

    let rodata_phys = add_pages_to_phys(TEXT_PHYS, text_pages)?;
    let data_phys = add_pages_to_phys(rodata_phys, rodata_pages)?;
    let heap_virt = add_pages_to_virt(layout.data_start, data_pages)?;
    let guard_virt = add_pages_to_virt(heap_virt, HEAP_RESERVED_PAGES)?;

    let mut mapper = aesynx_mm::PageTableMapper::<POLICY_TABLES, POLICY_MAPPED_FRAMES>::new()
        .map_err(KernelMappingSmokeError::Mapper)?;
    mapper
        .map_contiguous(
            layout.text_start,
            TEXT_PHYS,
            text_pages,
            aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadExecute),
        )
        .map_err(KernelMappingSmokeError::Mapper)?;
    mapper
        .map_contiguous(
            layout.rodata_start,
            rodata_phys,
            rodata_pages,
            aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadOnly),
        )
        .map_err(KernelMappingSmokeError::Mapper)?;
    mapper
        .map_contiguous(
            layout.data_start,
            data_phys,
            data_pages,
            aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadWrite),
        )
        .map_err(KernelMappingSmokeError::Mapper)?;

    let policy = aesynx_mm::KernelMappingPolicy::new(
        aesynx_mm::KernelVirtualRange::new(layout.text_start, text_pages),
        aesynx_mm::KernelVirtualRange::new(layout.rodata_start, rodata_pages),
        aesynx_mm::KernelVirtualRange::new(layout.data_start, data_pages),
        aesynx_mm::KernelVirtualRange::new(heap_virt, HEAP_RESERVED_PAGES),
        aesynx_mm::KernelVirtualRange::new(guard_virt, GUARD_PAGES),
        aesynx_mm::KernelVirtualRange::new(aesynx_abi::VirtAddr::new(0), 1),
    );
    let report = mapper
        .verify_kernel_mapping_policy(policy)
        .map_err(KernelMappingSmokeError::Mapper)?;

    let expected_mapped_pages = checked_add(checked_add(text_pages, rodata_pages)?, data_pages)?;
    let expected_reserved_pages = checked_add(HEAP_RESERVED_PAGES, GUARD_PAGES)?;
    if report.mapped_pages() != expected_mapped_pages
        || report.reserved_pages() != expected_reserved_pages
    {
        return Err(KernelMappingSmokeError::UnexpectedPolicy);
    }
    if !report.null_page_unmapped() {
        return Err(KernelMappingSmokeError::UnexpectedPolicy);
    }

    Ok(KernelMappingSmokeStatus {
        mapped_pages: report.mapped_pages(),
        reserved_pages: report.reserved_pages(),
        text_pages,
        rodata_pages,
        data_pages,
        text_rx_ok: true,
        rodata_read_only_ok: true,
        data_rw_nx_ok: true,
        heap_reserved_ok: true,
        guard_page_ok: true,
        null_page_ok: true,
    })
}

fn ensure_ordered_layout(
    layout: crate::kernel_sections::KernelSectionLayout,
) -> Result<(), KernelMappingSmokeError> {
    if layout.text_start.get() >= layout.text_end.get()
        || layout.text_end.get() > layout.rodata_start.get()
        || layout.rodata_start.get() >= layout.rodata_end.get()
        || layout.rodata_end.get() > layout.data_start.get()
        || layout.data_start.get() >= layout.data_end.get()
    {
        return Err(KernelMappingSmokeError::InvalidSectionLayout);
    }
    Ok(())
}

fn section_pages(
    start: aesynx_abi::VirtAddr,
    end: aesynx_abi::VirtAddr,
) -> Result<u64, KernelMappingSmokeError> {
    if start.get() >= end.get() || !page_aligned(start.get()) || !page_aligned(end.get()) {
        return Err(KernelMappingSmokeError::InvalidSectionLayout);
    }

    Ok((end.get() - start.get()) / aesynx_mm::FRAME_SIZE)
}

const fn page_aligned(value: u64) -> bool {
    value & (aesynx_mm::FRAME_SIZE - 1) == 0
}

fn add_pages_to_virt(
    virt: aesynx_abi::VirtAddr,
    pages: u64,
) -> Result<aesynx_abi::VirtAddr, KernelMappingSmokeError> {
    let offset = pages
        .checked_mul(aesynx_mm::FRAME_SIZE)
        .ok_or(KernelMappingSmokeError::AddressOverflow)?;
    virt.get()
        .checked_add(offset)
        .map(aesynx_abi::VirtAddr::new)
        .ok_or(KernelMappingSmokeError::AddressOverflow)
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

fn checked_add(left: u64, right: u64) -> Result<u64, KernelMappingSmokeError> {
    left.checked_add(right)
        .ok_or(KernelMappingSmokeError::AddressOverflow)
}
