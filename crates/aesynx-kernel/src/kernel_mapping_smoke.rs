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
    Plan(aesynx_kernel::kernel_mapping_policy::KernelMappingPlanError),
    AddressOverflow,
    UnexpectedPolicy,
}

const TEXT_PHYS: aesynx_abi::PhysAddr = aesynx_abi::PhysAddr::new(0x0030_0000);
const POLICY_TABLES: usize = aesynx_mm::PAGE_TABLE_LEVELS;
const POLICY_MAPPED_FRAMES: usize = 256;
const HEAP_RESERVED_PAGES: u64 = 2;
const GUARD_PAGES: u64 = 1;

pub fn run(
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

    Ok(KernelMappingSmokeStatus {
        mapped_pages: report.mapped_pages(),
        reserved_pages: report.reserved_pages(),
        text_pages: plan.text_pages(),
        rodata_pages: plan.rodata_pages(),
        data_pages: plan.data_pages(),
        text_rx_ok: report.text_rx(),
        rodata_read_only_ok: report.rodata_read_only(),
        data_rw_nx_ok: report.data_rw_nx(),
        heap_reserved_ok: report.reserved_heap_unmapped(),
        guard_page_ok: report.guard_page_unmapped(),
        null_page_ok: report.null_page_unmapped(),
    })
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
