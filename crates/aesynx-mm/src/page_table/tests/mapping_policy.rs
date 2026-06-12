use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageMapping, PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_no_executable_check_accepts_data_mappings_without_mutation() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
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

    mapper.ensure_no_executable_mappings()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_no_executable_check_accepts_empty_mapper() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.ensure_no_executable_mappings(), Ok(()));
    Ok(())
}

#[test]
fn mapper_no_executable_check_rejects_executable_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_no_executable_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_policy_checks_report_corruption_before_policy_violation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;
    mapper.mapped_pages = 2;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_no_executable_mappings(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_no_executable_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_no_executable_mappings(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_no_writable_check_accepts_read_only_mappings_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
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

    mapper.ensure_no_writable_mappings()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_no_writable_check_accepts_empty_mapper() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.ensure_no_writable_mappings(), Ok(()));
    Ok(())
}

#[test]
fn mapper_no_writable_check_rejects_writable_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_no_writable_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_no_writable_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_no_writable_mappings(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_no_device_check_accepts_normal_mappings_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
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

    mapper.ensure_no_device_mappings()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_no_device_check_accepts_empty_mapper() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.ensure_no_device_mappings(), Ok(()));
    Ok(())
}

#[test]
fn mapper_no_device_check_rejects_device_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly).device(),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_no_device_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_no_device_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_no_device_mappings(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_no_global_check_accepts_local_mappings_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
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

    mapper.ensure_no_global_mappings()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_no_global_check_accepts_empty_mapper() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.ensure_no_global_mappings(), Ok(()));
    Ok(())
}

#[test]
fn mapper_no_global_check_rejects_global_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let global_flags = GenericPageFlags::kernel(PageAccess::ReadOnly)
        .with_global()
        .map_err(|_error| PageTableError::InvalidMappingFlags)?;
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, global_flags)?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_no_global_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_no_global_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_no_global_mappings(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}
