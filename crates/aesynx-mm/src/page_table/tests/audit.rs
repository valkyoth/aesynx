use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{
    PAGE_TABLE_LEVELS, PageTableAudit, PageTableError, PageTableMapper, PageTableSlot,
};

#[test]
fn mapper_audit_rejects_empty_arena() {
    let mapper = PageTableMapper::<0> {
        tables: [],
        used: [],
        mapped_pages: 0,
    };

    assert_eq!(mapper.audit(), Err(PageTableError::EmptyArena));
}

#[test]
fn mapper_audit_reports_reachable_tables_and_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;

    let empty = mapper;
    assert_eq!(mapper.audit()?, PageTableAudit::new(4, 1, 1, 0));
    assert_eq!(mapper, empty);

    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let mapped = mapper;

    assert_eq!(
        mapper.audit()?,
        PageTableAudit::new(4, PAGE_TABLE_LEVELS as u64, PAGE_TABLE_LEVELS as u64, 1)
    );
    assert_eq!(mapper, mapped);
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
    let corrupt = mapper;

    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_audit_rejects_unreachable_used_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[1] = true;
    let corrupt = mapper;

    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_audit_rejects_unused_root_table() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[0] = false;
    let corrupt = mapper;

    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    assert_eq!(mapper, corrupt);
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
    let corrupt = mapper;

    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_audit_rejects_malformed_next_table_slot() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.tables[0].slots[0] = PageTableSlot { raw: 1 << 9 };
    let corrupt = mapper;

    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    assert_eq!(mapper, corrupt);
    Ok(())
}
