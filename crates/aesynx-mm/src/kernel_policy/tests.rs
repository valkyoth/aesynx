use alloc::format;

use aesynx_abi::{PhysAddr, VirtAddr};

use super::{KernelMappingPolicy, KernelVirtualRange};
use crate::{GenericPageFlags, PageAccess, PageTableError, PageTableMapper};

const TEXT: VirtAddr = VirtAddr::new(0xffff_9000_0000_0000);
const RODATA: VirtAddr = VirtAddr::new(0xffff_9000_0000_2000);
const DATA: VirtAddr = VirtAddr::new(0xffff_9000_0000_4000);
const HEAP: VirtAddr = VirtAddr::new(0xffff_9000_0000_6000);
const GUARD: VirtAddr = VirtAddr::new(0xffff_9000_0000_8000);
const TEXT_PHYS: PhysAddr = PhysAddr::new(0x0020_0000);
const RODATA_PHYS: PhysAddr = PhysAddr::new(0x0020_2000);
const DATA_PHYS: PhysAddr = PhysAddr::new(0x0020_4000);

fn policy() -> KernelMappingPolicy {
    KernelMappingPolicy::new(
        KernelVirtualRange::new(TEXT, 2),
        KernelVirtualRange::new(RODATA, 2),
        KernelVirtualRange::new(DATA, 2),
        KernelVirtualRange::new(HEAP, 2),
        KernelVirtualRange::new(GUARD, 1),
        KernelVirtualRange::new(VirtAddr::new(0), 1),
    )
}

fn mapper_with_policy() -> Result<PageTableMapper<8>, PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_contiguous(
        TEXT,
        TEXT_PHYS,
        2,
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;
    mapper.map_contiguous(
        RODATA,
        RODATA_PHYS,
        2,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_contiguous(
        DATA,
        DATA_PHYS,
        2,
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;
    Ok(mapper)
}

#[test]
fn kernel_mapping_policy_accepts_expected_layout() -> Result<(), PageTableError> {
    let mapper = mapper_with_policy()?;

    let report = mapper.verify_kernel_mapping_policy(policy())?;

    assert_eq!(report.mapped_pages(), 6);
    assert_eq!(report.reserved_pages(), 3);
    assert!(report.text_rx());
    assert!(report.rodata_read_only());
    assert!(report.data_rw_nx());
    assert!(report.reserved_heap_unmapped());
    assert!(report.guard_page_unmapped());
    assert!(report.null_page_unmapped());
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_writable_text() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_policy()?;
    mapper.protect_contiguous(TEXT, 2, GenericPageFlags::kernel(PageAccess::ReadWrite))?;

    assert_eq!(
        mapper.verify_kernel_mapping_policy(policy()),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_executable_data() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_policy()?;
    mapper.protect_contiguous(DATA, 2, GenericPageFlags::kernel(PageAccess::ReadExecute))?;

    assert_eq!(
        mapper.verify_kernel_mapping_policy(policy()),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_writable_rodata() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_policy()?;
    mapper.protect_contiguous(RODATA, 2, GenericPageFlags::kernel(PageAccess::ReadWrite))?;

    assert_eq!(
        mapper.verify_kernel_mapping_policy(policy()),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_executable_rodata() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_policy()?;
    mapper.protect_contiguous(RODATA, 2, GenericPageFlags::kernel(PageAccess::ReadExecute))?;

    assert_eq!(
        mapper.verify_kernel_mapping_policy(policy()),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_user_accessible_text() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_policy()?;
    mapper.protect_contiguous(TEXT, 2, GenericPageFlags::user(PageAccess::ReadExecute))?;

    assert_eq!(
        mapper.verify_kernel_mapping_policy(policy()),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_device_data() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_policy()?;
    mapper.protect_contiguous(
        DATA,
        2,
        GenericPageFlags::kernel(PageAccess::ReadWrite).device(),
    )?;

    assert_eq!(
        mapper.verify_kernel_mapping_policy(policy()),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_global_data() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_policy()?;
    let global_data = match GenericPageFlags::kernel(PageAccess::ReadWrite).with_global() {
        Ok(flags) => flags,
        Err(_error) => return Err(PageTableError::InvalidMappingFlags),
    };
    mapper.protect_contiguous(DATA, 2, global_data)?;

    assert_eq!(
        mapper.verify_kernel_mapping_policy(policy()),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_mapped_reserved_heap() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_policy()?;
    mapper.map_page(
        HEAP,
        PhysAddr::new(0x0020_6000),
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;

    assert_eq!(
        mapper.verify_kernel_mapping_policy(policy()),
        Err(PageTableError::AlreadyMapped)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_mapped_guard_page() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_policy()?;
    mapper.map_page(
        GUARD,
        PhysAddr::new(0x0020_8000),
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    assert_eq!(
        mapper.verify_kernel_mapping_policy(policy()),
        Err(PageTableError::AlreadyMapped)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_mapped_null_page() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_policy()?;
    mapper.map_page(
        VirtAddr::new(0),
        PhysAddr::new(0x0020_9000),
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    assert_eq!(
        mapper.verify_kernel_mapping_policy(policy()),
        Err(PageTableError::AlreadyMapped)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_overlapping_ranges() -> Result<(), PageTableError> {
    let mapper = mapper_with_policy()?;
    let overlapping = KernelMappingPolicy::new(
        KernelVirtualRange::new(TEXT, 2),
        KernelVirtualRange::new(VirtAddr::new(TEXT.get() + crate::FRAME_SIZE), 2),
        KernelVirtualRange::new(DATA, 2),
        KernelVirtualRange::new(HEAP, 2),
        KernelVirtualRange::new(GUARD, 1),
        KernelVirtualRange::new(VirtAddr::new(0), 1),
    );

    assert_eq!(
        mapper.verify_kernel_mapping_policy(overlapping),
        Err(PageTableError::UnexpectedVirtualAddressSpace)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_zero_page_ranges() -> Result<(), PageTableError> {
    let mapper = mapper_with_policy()?;
    let zero_text = KernelMappingPolicy::new(
        KernelVirtualRange::new(TEXT, 0),
        KernelVirtualRange::new(RODATA, 2),
        KernelVirtualRange::new(DATA, 2),
        KernelVirtualRange::new(HEAP, 2),
        KernelVirtualRange::new(GUARD, 1),
        KernelVirtualRange::new(VirtAddr::new(0), 1),
    );

    assert_eq!(
        mapper.verify_kernel_mapping_policy(zero_text),
        Err(PageTableError::InvalidPageCount)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_overflowing_ranges() -> Result<(), PageTableError> {
    let mapper = mapper_with_policy()?;
    let overflowing_text = KernelMappingPolicy::new(
        KernelVirtualRange::new(VirtAddr::new(u64::MAX - crate::FRAME_SIZE), 2),
        KernelVirtualRange::new(RODATA, 2),
        KernelVirtualRange::new(DATA, 2),
        KernelVirtualRange::new(HEAP, 2),
        KernelVirtualRange::new(GUARD, 1),
        KernelVirtualRange::new(VirtAddr::new(0), 1),
    );

    assert_eq!(
        mapper.verify_kernel_mapping_policy(overflowing_text),
        Err(PageTableError::AddressOverflow)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_low_half_reserved_heap() -> Result<(), PageTableError> {
    let mapper = mapper_with_policy()?;
    let low_heap = KernelMappingPolicy::new(
        KernelVirtualRange::new(TEXT, 2),
        KernelVirtualRange::new(RODATA, 2),
        KernelVirtualRange::new(DATA, 2),
        KernelVirtualRange::new(VirtAddr::new(crate::FRAME_SIZE), 1),
        KernelVirtualRange::new(GUARD, 1),
        KernelVirtualRange::new(VirtAddr::new(0), 1),
    );

    assert_eq!(
        mapper.verify_kernel_mapping_policy(low_heap),
        Err(PageTableError::UnexpectedVirtualAddressSpace)
    );
    Ok(())
}

#[test]
fn kernel_mapping_policy_rejects_bad_null_page_descriptor() -> Result<(), PageTableError> {
    let mapper = mapper_with_policy()?;
    let bad_null = KernelMappingPolicy::new(
        KernelVirtualRange::new(TEXT, 2),
        KernelVirtualRange::new(RODATA, 2),
        KernelVirtualRange::new(DATA, 2),
        KernelVirtualRange::new(HEAP, 2),
        KernelVirtualRange::new(GUARD, 1),
        KernelVirtualRange::new(VirtAddr::new(crate::FRAME_SIZE), 1),
    );

    assert_eq!(
        mapper.verify_kernel_mapping_policy(bad_null),
        Err(PageTableError::UnexpectedVirtualAddressSpace)
    );
    Ok(())
}

#[test]
fn kernel_virtual_range_debug_redacts_start_address() {
    let debug = format!("{:?}", KernelVirtualRange::new(TEXT, 2));

    assert!(debug.contains("KernelVirtualRange"));
    assert!(debug.contains("pages: 2"));
    assert!(!debug.contains("ffff_9000"));
}
