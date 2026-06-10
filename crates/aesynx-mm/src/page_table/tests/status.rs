use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PAGE_TABLE_LEVELS, PageTableError, PageTableMapper};

#[test]
fn mapper_checked_status_matches_valid_mapper() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.status_checked()?, mapper.status());

    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    let status = mapper.status_checked()?;
    assert_eq!(status.total_tables, 4);
    assert_eq!(status.used_tables, PAGE_TABLE_LEVELS as u64);
    assert_eq!(status.mapped_pages, 1);
    assert_eq!(status, mapper.status());
    Ok(())
}

#[test]
fn mapper_checked_status_rejects_accounting_drift() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    mapper.mapped_pages = 0;

    assert_eq!(mapper.status().mapped_pages, 0);
    assert_eq!(mapper.status_checked(), Err(PageTableError::CorruptTable));
    Ok(())
}
