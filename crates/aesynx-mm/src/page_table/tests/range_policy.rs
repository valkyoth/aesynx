use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{
    PAGE_TABLE_ENTRIES, PageMapping, PageTableError, PageTableMapper, PageTableSlot,
};

#[test]
fn mapper_verifies_write_protected_range_without_physical_contiguity() -> Result<(), PageTableError>
{
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
    let before = mapper;

    mapper.ensure_write_protected_contiguous(first_virt, 2)?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_verifies_non_executable_range_without_physical_contiguity() -> Result<(), PageTableError>
{
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
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;
    let before = mapper;

    mapper.ensure_non_executable_contiguous(first_virt, 2)?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_verifies_executable_range_without_physical_contiguity() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let first_virt = KERNEL_VIRT;
    let second_virt = VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE);
    mapper.map_page(
        first_virt,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;
    mapper.map_page(
        second_virt,
        PhysAddr::new(KERNEL_PHYS.get() + 0x3000),
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;
    let before = mapper;

    mapper.ensure_executable_contiguous(first_virt, 2)?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_verifies_normal_memory_range_without_physical_contiguity() -> Result<(), PageTableError> {
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
    let before = mapper;

    mapper.ensure_normal_memory_contiguous(first_virt, 2)?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_verifies_local_range_without_physical_contiguity() -> Result<(), PageTableError> {
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
    let before = mapper;

    mapper.ensure_local_contiguous(first_virt, 2)?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_write_protected_range_check_rejects_writable_pages() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_write_protected_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_non_executable_range_check_rejects_executable_pages() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_non_executable_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_executable_range_check_rejects_non_executable_pages() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_executable_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_normal_memory_range_check_rejects_device_pages() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        GenericPageFlags::kernel(PageAccess::ReadOnly).device(),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_normal_memory_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_local_range_check_rejects_global_pages() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let global = GenericPageFlags::kernel(PageAccess::ReadOnly)
        .with_global()
        .map_err(|_error| PageTableError::InvalidMappingFlags)?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        global,
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_local_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_write_protected_range_check_rejects_gaps_and_invalid_ranges() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let max_pages = (4 * PAGE_TABLE_ENTRIES) as u64;

    assert_eq!(
        mapper.ensure_write_protected_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        mapper.ensure_write_protected_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_write_protected_contiguous(KERNEL_VIRT, max_pages + 1),
        Err(PageTableError::RangeTooLarge)
    );
    Ok(())
}

#[test]
fn mapper_non_executable_range_check_rejects_gaps_and_invalid_ranges() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let max_pages = (4 * PAGE_TABLE_ENTRIES) as u64;

    assert_eq!(
        mapper.ensure_non_executable_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        mapper.ensure_non_executable_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_non_executable_contiguous(KERNEL_VIRT, max_pages + 1),
        Err(PageTableError::RangeTooLarge)
    );
    Ok(())
}

#[test]
fn mapper_executable_range_check_rejects_gaps_and_invalid_ranges() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;
    let max_pages = (4 * PAGE_TABLE_ENTRIES) as u64;

    assert_eq!(
        mapper.ensure_executable_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        mapper.ensure_executable_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_executable_contiguous(KERNEL_VIRT, max_pages + 1),
        Err(PageTableError::RangeTooLarge)
    );
    Ok(())
}

#[test]
fn mapper_normal_memory_range_check_rejects_gaps_and_invalid_ranges() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let max_pages = (4 * PAGE_TABLE_ENTRIES) as u64;

    assert_eq!(
        mapper.ensure_normal_memory_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        mapper.ensure_normal_memory_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_normal_memory_contiguous(KERNEL_VIRT, max_pages + 1),
        Err(PageTableError::RangeTooLarge)
    );
    Ok(())
}

#[test]
fn mapper_local_range_check_rejects_gaps_and_invalid_ranges() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let max_pages = (4 * PAGE_TABLE_ENTRIES) as u64;

    assert_eq!(
        mapper.ensure_local_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        mapper.ensure_local_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_local_contiguous(KERNEL_VIRT, max_pages + 1),
        Err(PageTableError::RangeTooLarge)
    );
    Ok(())
}

#[test]
fn mapper_write_protected_range_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_write_protected_contiguous(VirtAddr::new(0), 1),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_non_executable_range_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_non_executable_contiguous(VirtAddr::new(0), 1),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_executable_range_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_executable_contiguous(VirtAddr::new(0), 1),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_normal_memory_range_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_normal_memory_contiguous(VirtAddr::new(0), 1),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_local_range_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_local_contiguous(VirtAddr::new(0), 1),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}
