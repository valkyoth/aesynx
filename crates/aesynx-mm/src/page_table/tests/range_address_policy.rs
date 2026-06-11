use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::range::{VirtualSpace, validate_virtual_space};
use crate::page_table::{PAGE_TABLE_ENTRIES, PageTableError, PageTableMapper, PageTableSlot};

const USER_VIRT: VirtAddr = VirtAddr::new(0x0000_0000_0040_0000);

#[test]
fn mapper_verifies_kernel_space_range() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, flags)?;
    let before = mapper;

    mapper.ensure_kernel_space_contiguous(KERNEL_VIRT, 2)?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_verifies_user_space_range() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::user(PageAccess::ReadOnly);
    mapper.map_contiguous(USER_VIRT, KERNEL_PHYS, 2, flags)?;
    let before = mapper;

    mapper.ensure_user_space_contiguous(USER_VIRT, 2)?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn virtual_space_validator_rejects_zero_pages() {
    assert_eq!(
        validate_virtual_space(KERNEL_VIRT, 0, VirtualSpace::Kernel),
        Err(PageTableError::InvalidPageCount)
    );
}

#[test]
fn mapper_kernel_space_range_rejects_low_half_or_user_flags() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let kernel_flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let user_flags = GenericPageFlags::user(PageAccess::ReadOnly);
    mapper.map_contiguous(USER_VIRT, KERNEL_PHYS, 2, kernel_flags)?;
    mapper.map_contiguous(
        KERNEL_VIRT,
        PhysAddr::new(KERNEL_PHYS.get() + 0x4000),
        2,
        user_flags,
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_kernel_space_contiguous(USER_VIRT, 2),
        Err(PageTableError::UnexpectedVirtualAddressSpace)
    );
    assert_eq!(
        mapper.ensure_kernel_space_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_user_space_range_rejects_high_half_or_kernel_flags() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let kernel_flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let user_flags = GenericPageFlags::user(PageAccess::ReadOnly);
    mapper.map_contiguous(USER_VIRT, KERNEL_PHYS, 2, kernel_flags)?;
    mapper.map_contiguous(
        KERNEL_VIRT,
        PhysAddr::new(KERNEL_PHYS.get() + 0x4000),
        2,
        user_flags,
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_user_space_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::UnexpectedVirtualAddressSpace)
    );
    assert_eq!(
        mapper.ensure_user_space_contiguous(USER_VIRT, 2),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_space_range_checks_reject_gaps_and_invalid_ranges() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let user_flags = GenericPageFlags::user(PageAccess::ReadOnly);
    mapper.map_page(USER_VIRT, KERNEL_PHYS, user_flags)?;
    let before = mapper;
    let max_pages = (8 * PAGE_TABLE_ENTRIES) as u64;

    assert_eq!(
        mapper.ensure_user_space_contiguous(USER_VIRT, 2),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        mapper.ensure_user_space_contiguous(USER_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_user_space_contiguous(USER_VIRT, max_pages + 1),
        Err(PageTableError::RangeTooLarge)
    );
    assert_eq!(
        mapper.ensure_kernel_space_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_kernel_space_contiguous(KERNEL_VIRT, max_pages + 1),
        Err(PageTableError::RangeTooLarge)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_space_range_checks_reject_corrupt_tables() -> Result<(), PageTableError> {
    let mut kernel_mapper = PageTableMapper::<4>::new()?;
    kernel_mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    assert_eq!(
        kernel_mapper.ensure_kernel_space_contiguous(VirtAddr::new(0), 1),
        Err(PageTableError::UnexpectedVirtualAddressSpace)
    );
    kernel_mapper.used[1] = false;
    assert_eq!(
        kernel_mapper.ensure_kernel_space_contiguous(KERNEL_VIRT, 1),
        Err(PageTableError::CorruptTable)
    );

    let mut user_mapper = PageTableMapper::<4>::new()?;
    user_mapper.tables[0].slots[0] = PageTableSlot::next(1)?;
    assert_eq!(
        user_mapper.ensure_user_space_contiguous(USER_VIRT, 1),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}
