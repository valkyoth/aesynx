use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{
    PAGE_TABLE_ENTRIES, PageMapping, PageTableError, PageTableMapper, PageTableSlot,
};

#[test]
fn mapper_verifies_kernel_mapped_range_without_physical_contiguity() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let first_virt = KERNEL_VIRT;
    let second_virt = VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE);
    mapper.map_page(
        first_virt,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        second_virt,
        PhysAddr::new(KERNEL_PHYS.get() + 0x3000),
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;
    let before = mapper.clone();

    mapper.ensure_kernel_mapped_contiguous(first_virt, 2)?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_verifies_user_mapped_range_without_physical_contiguity() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let first_virt = KERNEL_VIRT;
    let second_virt = VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE);
    mapper.map_page(
        first_virt,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        second_virt,
        PhysAddr::new(KERNEL_PHYS.get() + 0x3000),
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    let before = mapper.clone();

    mapper.ensure_user_mapped_contiguous(first_virt, 2)?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_kernel_range_check_rejects_user_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    let before = mapper.clone();

    assert_eq!(
        mapper.ensure_kernel_mapped_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_user_range_check_rejects_kernel_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper.clone();

    assert_eq!(
        mapper.ensure_user_mapped_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_kernel_range_check_rejects_gaps_and_invalid_ranges() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper.clone();
    let max_pages = (4 * PAGE_TABLE_ENTRIES) as u64;

    assert_eq!(
        mapper.ensure_kernel_mapped_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        mapper.ensure_kernel_mapped_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_kernel_mapped_contiguous(KERNEL_VIRT, max_pages + 1),
        Err(PageTableError::RangeTooLarge)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_user_range_check_rejects_gaps_and_invalid_ranges() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    let before = mapper.clone();
    let max_pages = (4 * PAGE_TABLE_ENTRIES) as u64;

    assert_eq!(
        mapper.ensure_user_mapped_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        mapper.ensure_user_mapped_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_user_mapped_contiguous(KERNEL_VIRT, max_pages + 1),
        Err(PageTableError::RangeTooLarge)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_kernel_range_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    let corrupt = mapper.clone();

    assert_eq!(
        mapper.ensure_kernel_mapped_contiguous(VirtAddr::new(0), 1),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_user_range_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    let corrupt = mapper.clone();

    assert_eq!(
        mapper.ensure_user_mapped_contiguous(VirtAddr::new(0), 1),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}
