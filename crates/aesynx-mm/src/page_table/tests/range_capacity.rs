use crate::page_table::{DEFAULT_MAPPED_FRAME_INDEX_ENTRIES, PageTableError, PageTableMapper};
use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};

#[test]
fn mapper_explicit_frame_index_capacity_allows_larger_ranges() -> Result<(), PageTableError> {
    const MAX_MAPPED: usize = DEFAULT_MAPPED_FRAME_INDEX_ENTRIES + 1;

    let mut mapper = PageTableMapper::<4, MAX_MAPPED>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let pages = MAX_MAPPED as u64;

    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, pages, flags)?;

    assert_eq!(mapper.status().mapped_pages(), pages);
    assert_eq!(
        mapper.mapping_for_contiguous(KERNEL_VIRT, pages)?.pages(),
        pages
    );
    Ok(())
}
