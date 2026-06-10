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
