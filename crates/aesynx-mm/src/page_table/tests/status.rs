use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PAGE_TABLE_LEVELS, PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_checked_status_matches_valid_mapper() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;

    let empty = mapper.clone();
    assert_eq!(mapper.status_checked()?, mapper.status());
    assert_eq!(mapper, empty);

    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let mapped = mapper.clone();

    let status = mapper.status_checked()?;
    assert_eq!(status.total_tables(), 4);
    assert_eq!(status.used_tables(), PAGE_TABLE_LEVELS as u64);
    assert_eq!(status.mapped_pages(), 1);
    assert_eq!(status, mapper.status());
    assert_eq!(mapper, mapped);
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
    let corrupt = mapper.clone();

    assert_eq!(mapper.status().mapped_pages(), 0);
    assert_eq!(mapper.status_checked(), Err(PageTableError::CorruptTable));
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_checked_status_rejects_unreachable_used_tables_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[1] = true;
    let corrupt = mapper.clone();

    assert_eq!(mapper.status_checked(), Err(PageTableError::CorruptTable));
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_checked_status_rejects_duplicate_table_parent_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.tables[0].slots[1] = PageTableSlot::next(1)?;
    let corrupt = mapper.clone();

    assert_eq!(mapper.status_checked(), Err(PageTableError::CorruptTable));
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_checked_status_rejects_empty_arena() {
    let mapper = PageTableMapper::<0> {
        tables: [],
        used: [],
        mapped_pages: 0,
        mapped_frames: crate::page_table::MappedFrameIndex::empty(),
    };

    assert_eq!(mapper.status_checked(), Err(PageTableError::EmptyArena));
}
