use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageTableError, PageTableMapper};

#[test]
fn mapper_checked_root_matches_valid_mapper() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.root_table_checked()?, mapper.root_table());

    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    assert_eq!(mapper.root_table_checked()?.table_index(), 0);
    Ok(())
}

#[test]
fn mapper_checked_root_rejects_corrupt_mapper() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[0] = false;

    assert_eq!(mapper.root_table().table_index(), 0);
    assert_eq!(
        mapper.root_table_checked(),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}
