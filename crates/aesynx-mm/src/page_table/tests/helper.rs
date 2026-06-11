use crate::page_table::{PAGE_TABLE_ENTRIES, PageTableError, PageTableMapper, PageTableSlot};

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
