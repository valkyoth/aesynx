use aesynx_abi::VirtAddr;

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageMapping, PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_accepts_kernel_only_mappings_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    let before = mapper;

    mapper.ensure_no_user_mappings()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_kernel_only_check_accepts_empty_mapper() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.ensure_no_user_mappings(), Ok(()));
    Ok(())
}

#[test]
fn mapper_kernel_only_check_rejects_user_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;

    assert_eq!(
        mapper.ensure_no_user_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn mapper_kernel_only_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;

    assert_eq!(
        mapper.ensure_no_user_mappings(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.mapping_for_page(VirtAddr::new(0)),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}

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
        aesynx_abi::PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
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

    assert_eq!(
        mapper.ensure_no_executable_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn mapper_no_executable_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;

    assert_eq!(
        mapper.ensure_no_executable_mappings(),
        Err(PageTableError::CorruptTable)
    );
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
        aesynx_abi::PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
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

    assert_eq!(
        mapper.ensure_no_writable_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn mapper_no_writable_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;

    assert_eq!(
        mapper.ensure_no_writable_mappings(),
        Err(PageTableError::CorruptTable)
    );
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
        aesynx_abi::PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
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

    assert_eq!(
        mapper.ensure_no_device_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn mapper_no_device_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;

    assert_eq!(
        mapper.ensure_no_device_mappings(),
        Err(PageTableError::CorruptTable)
    );
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
        aesynx_abi::PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
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

    assert_eq!(
        mapper.ensure_no_global_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn mapper_no_global_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;

    assert_eq!(
        mapper.ensure_no_global_mappings(),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}
