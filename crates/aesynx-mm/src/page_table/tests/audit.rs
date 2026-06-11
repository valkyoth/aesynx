use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{
    PAGE_TABLE_LEVELS, PageTableAudit, PageTableError, PageTableMapper, PageTableSlot,
};

#[test]
fn mapper_audit_reports_reachable_tables_and_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.audit()?, PageTableAudit::new(4, 1, 1, 0));

    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    assert_eq!(
        mapper.audit()?,
        PageTableAudit::new(4, PAGE_TABLE_LEVELS as u64, PAGE_TABLE_LEVELS as u64, 1)
    );
    Ok(())
}

#[test]
fn mapper_audit_rejects_mapping_accounting_drift() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 0;

    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    Ok(())
}

#[test]
fn mapper_audit_rejects_unreachable_used_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[1] = true;

    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    Ok(())
}

#[test]
fn mapper_audit_rejects_unused_root_table() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[0] = false;

    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    Ok(())
}

#[test]
fn mapper_audit_rejects_duplicate_table_parent() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.tables[0].slots[1] = PageTableSlot::next(1)?;

    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    Ok(())
}

#[test]
fn mapper_audit_rejects_malformed_next_table_slot() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.tables[0].slots[0] = PageTableSlot { raw: 1 << 9 };

    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    Ok(())
}
