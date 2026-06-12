use aesynx_abi::VirtAddr;

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageMapping, PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_no_user_space_check_accepts_kernel_half_mappings_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    mapper.ensure_no_user_space_mappings()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_no_user_space_check_accepts_empty_mapper() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.ensure_no_user_space_mappings(), Ok(()));
    Ok(())
}

#[test]
fn mapper_no_user_space_check_rejects_low_half_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        VirtAddr::new(0x0000_0000_0040_0000),
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_no_user_space_mappings(),
        Err(PageTableError::UnexpectedVirtualAddressSpace)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_no_user_space_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_no_user_space_mappings(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

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
    let before = mapper;

    assert_eq!(
        mapper.ensure_no_user_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_kernel_only_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_no_user_mappings(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.mapping_for_page(VirtAddr::new(0)),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_kernel_user_guard_accepts_low_half_user_mappings_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        VirtAddr::new(0x0000_0000_0040_0000),
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    mapper.ensure_no_kernel_space_user_mappings()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_kernel_user_guard_accepts_high_half_kernel_mappings_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    mapper.ensure_no_kernel_space_user_mappings()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_kernel_user_guard_accepts_empty_mapper() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.ensure_no_kernel_space_user_mappings(), Ok(()));
    Ok(())
}

#[test]
fn mapper_kernel_user_guard_rejects_high_half_user_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_no_kernel_space_user_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_kernel_user_guard_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_no_kernel_space_user_mappings(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_user_kernel_guard_accepts_low_half_user_mappings_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        VirtAddr::new(0x0000_0000_0040_0000),
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    mapper.ensure_no_user_space_kernel_mappings()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_user_kernel_guard_accepts_high_half_kernel_mappings_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    mapper.ensure_no_user_space_kernel_mappings()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_user_kernel_guard_accepts_empty_mapper() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.ensure_no_user_space_kernel_mappings(), Ok(()));
    Ok(())
}

#[test]
fn mapper_user_kernel_guard_rejects_low_half_kernel_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        VirtAddr::new(0x0000_0000_0040_0000),
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_no_user_space_kernel_mappings(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_user_kernel_guard_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;
    let corrupt = mapper;

    assert_eq!(
        mapper.ensure_no_user_space_kernel_mappings(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}
