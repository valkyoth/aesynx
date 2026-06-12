use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_checked_root_matches_valid_mapper() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;

    let empty = mapper;
    assert_eq!(mapper.root_table_checked()?, mapper.root_table());
    assert_eq!(mapper, empty);

    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let mapped = mapper;

    assert_eq!(mapper.root_table_checked()?.table_index(), 0);
    assert_eq!(mapper, mapped);
    Ok(())
}

#[test]
fn mapper_checked_root_rejects_corrupt_mapper() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[0] = false;
    let corrupt = mapper;

    assert_eq!(mapper.root_table().table_index(), 0);
    assert_eq!(
        mapper.root_table_checked(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_checked_root_rejects_duplicate_table_parent_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.tables[0].slots[1] = PageTableSlot::next(1)?;
    let corrupt = mapper;

    assert_eq!(
        mapper.root_table_checked(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_checked_root_rejects_empty_arena() {
    let mapper = PageTableMapper::<0> {
        tables: [],
        used: [],
        mapped_pages: 0,
        mapped_frames: crate::page_table::MappedFrameIndex::empty(),
    };

    assert_eq!(mapper.root_table_checked(), Err(PageTableError::EmptyArena));
}
