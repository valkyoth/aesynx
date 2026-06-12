#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelMappingSmokeStatus {
    pub mapped_pages: u64,
    pub reserved_pages: u64,
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
    UnexpectedPolicy,
}

const TEXT_VIRT: aesynx_abi::VirtAddr = aesynx_abi::VirtAddr::new(0xffff_9100_0000_0000);
const RODATA_VIRT: aesynx_abi::VirtAddr = aesynx_abi::VirtAddr::new(0xffff_9100_0000_2000);
const DATA_VIRT: aesynx_abi::VirtAddr = aesynx_abi::VirtAddr::new(0xffff_9100_0000_4000);
const HEAP_VIRT: aesynx_abi::VirtAddr = aesynx_abi::VirtAddr::new(0xffff_9100_0000_6000);
const GUARD_VIRT: aesynx_abi::VirtAddr = aesynx_abi::VirtAddr::new(0xffff_9100_0000_8000);
const TEXT_PHYS: aesynx_abi::PhysAddr = aesynx_abi::PhysAddr::new(0x0030_0000);
const RODATA_PHYS: aesynx_abi::PhysAddr = aesynx_abi::PhysAddr::new(0x0030_2000);
const DATA_PHYS: aesynx_abi::PhysAddr = aesynx_abi::PhysAddr::new(0x0030_4000);
const POLICY_TABLES: usize = aesynx_mm::PAGE_TABLE_LEVELS;

pub fn run() -> Result<KernelMappingSmokeStatus, KernelMappingSmokeError> {
    let mut mapper = aesynx_mm::PageTableMapper::<POLICY_TABLES>::new()
        .map_err(KernelMappingSmokeError::Mapper)?;
    mapper
        .map_contiguous(
            TEXT_VIRT,
            TEXT_PHYS,
            2,
            aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadExecute),
        )
        .map_err(KernelMappingSmokeError::Mapper)?;
    mapper
        .map_contiguous(
            RODATA_VIRT,
            RODATA_PHYS,
            2,
            aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadOnly),
        )
        .map_err(KernelMappingSmokeError::Mapper)?;
    mapper
        .map_contiguous(
            DATA_VIRT,
            DATA_PHYS,
            2,
            aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadWrite),
        )
        .map_err(KernelMappingSmokeError::Mapper)?;

    let policy = aesynx_mm::KernelMappingPolicy::new(
        aesynx_mm::KernelVirtualRange::new(TEXT_VIRT, 2),
        aesynx_mm::KernelVirtualRange::new(RODATA_VIRT, 2),
        aesynx_mm::KernelVirtualRange::new(DATA_VIRT, 2),
        aesynx_mm::KernelVirtualRange::new(HEAP_VIRT, 2),
        aesynx_mm::KernelVirtualRange::new(GUARD_VIRT, 1),
        aesynx_mm::KernelVirtualRange::new(aesynx_abi::VirtAddr::new(0), 1),
    );
    let report = mapper
        .verify_kernel_mapping_policy(policy)
        .map_err(KernelMappingSmokeError::Mapper)?;

    if report.mapped_pages() != 6 || report.reserved_pages() != 3 {
        return Err(KernelMappingSmokeError::UnexpectedPolicy);
    }
    if !report.null_page_unmapped() {
        return Err(KernelMappingSmokeError::UnexpectedPolicy);
    }

    Ok(KernelMappingSmokeStatus {
        mapped_pages: report.mapped_pages(),
        reserved_pages: report.reserved_pages(),
        text_rx_ok: true,
        rodata_read_only_ok: true,
        data_rw_nx_ok: true,
        heap_reserved_ok: true,
        guard_page_ok: true,
        null_page_ok: true,
    })
}
