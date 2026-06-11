use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{
    PAGE_TABLE_ENTRIES, PAGE_TABLE_LEVELS, PageTableError, PageTableMapper, PageTableSlot,
};

#[test]
fn mapper_next_table_helper_rejects_unused_child_link() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.tables[0].slots[0] = PageTableSlot::next(1)?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_next_table(0, 0),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_next_table_helper_rejects_out_of_range_child_link() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<2>::new()?;
    mapper.tables[0].slots[0] = PageTableSlot {
        raw: 1 | (1 << 9) | (2 << 12),
    };
    let before = mapper;

    assert_eq!(
        mapper.ensure_next_table(0, 0),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_next_table_helper_rejects_invalid_parent_index() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<2>::new()?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_next_table(2, 0),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_next_table_helper_rejects_invalid_slot_index() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<2>::new()?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_next_table(0, PAGE_TABLE_ENTRIES),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_reclaim_helper_rejects_root_child_path_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let before = mapper;
    let indices = [0usize; PAGE_TABLE_LEVELS];
    let path = [0usize; PAGE_TABLE_LEVELS];

    assert_eq!(
        mapper.reclaim_empty_tables(indices, path),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_reclaim_helper_rejects_out_of_range_path_without_mutation() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    let before = mapper;
    let indices = [0usize; PAGE_TABLE_LEVELS];
    let path = [0usize, 1, 2, 4];

    assert_eq!(
        mapper.reclaim_empty_tables(indices, path),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_reclaim_helper_rejects_invalid_parent_slot_without_mutation() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let mut indices = [0usize; PAGE_TABLE_LEVELS];
    indices[PAGE_TABLE_LEVELS - 2] = PAGE_TABLE_ENTRIES;
    let path = [0usize, 1, 2, 3];
    let before = mapper;

    assert_eq!(
        mapper.reclaim_empty_tables(indices, path),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_reclaim_helper_rejects_mismatched_parent_link() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<5>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.tables[1].slots[0] = PageTableSlot::next(4)?;
    mapper.used[4] = true;
    let mut slot = 0usize;
    while slot < PAGE_TABLE_ENTRIES {
        mapper.tables[3].slots[slot] = PageTableSlot::EMPTY;
        slot += 1;
    }
    let before = mapper;
    let indices = [0usize; PAGE_TABLE_LEVELS];
    let path = [0usize, 1, 2, 3];

    assert_eq!(
        mapper.reclaim_empty_tables(indices, path),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_reclaim_helper_rejects_unused_path_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let before = mapper;
    let indices = [0usize; PAGE_TABLE_LEVELS];
    let path = [0usize, 1, 2, 3];

    assert_eq!(
        mapper.reclaim_empty_tables(indices, path),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_reclaim_helper_rejects_late_invalid_path_atomically() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let mut slot = 0usize;
    while slot < PAGE_TABLE_ENTRIES {
        mapper.tables[3].slots[slot] = PageTableSlot::EMPTY;
        slot += 1;
    }
    let before = mapper;
    let indices = [0usize; PAGE_TABLE_LEVELS];
    let path = [0usize, 4, 2, 3];

    assert_eq!(
        mapper.reclaim_empty_tables(indices, path),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}
