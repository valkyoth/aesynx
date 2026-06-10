use aesynx_abi::VirtAddr;

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_alias_check_accepts_empty_mapper() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    mapper.ensure_no_physical_aliases()?;

    assert_eq!(mapper.status().mapped_pages, 0);
    Ok(())
}

#[test]
fn mapper_alias_check_accepts_distinct_frames_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 3, flags)?;
    let before = mapper;

    mapper.ensure_no_physical_aliases()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_alias_check_rejects_duplicate_physical_frames() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let alias_virt = VirtAddr::new(KERNEL_VIRT.get() + 0x4000);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mapper.map_page(alias_virt, KERNEL_PHYS, flags)?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_no_physical_aliases(),
        Err(PageTableError::PhysicalAlias)
    );
    assert_eq!(mapper, before);
    assert_eq!(mapper.status().mapped_pages, 2);
    Ok(())
}

#[test]
fn mapper_alias_check_rejects_aliases_with_different_flags() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let alias_virt = VirtAddr::new(KERNEL_VIRT.get() + 0x4000);
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        alias_virt,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;

    assert_eq!(
        mapper.ensure_no_physical_aliases(),
        Err(PageTableError::PhysicalAlias)
    );
    Ok(())
}

#[test]
fn mapper_alias_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.tables[0].slots[0] = PageTableSlot::next(1)?;

    assert_eq!(
        mapper.ensure_no_physical_aliases(),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}

#[test]
fn mapper_alias_check_rejects_accounting_drift() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 2;

    assert_eq!(
        mapper.ensure_no_physical_aliases(),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}
