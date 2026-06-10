use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{
    PAGE_TABLE_ENTRIES, PageMapping, PageTableError, PageTableMapper, PageTableSlot,
};

#[test]
fn mapper_verifies_mapped_contiguous_range_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let first_virt = KERNEL_VIRT;
    let second_virt = VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE);
    mapper.map_page(
        first_virt,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        second_virt,
        PhysAddr::new(KERNEL_PHYS.get() + 0x3000),
        GenericPageFlags::kernel(PageAccess::ReadExecute),
    )?;
    let before = mapper;

    mapper.ensure_mapped_contiguous(first_virt, 2)?;

    assert_eq!(mapper, before);
    assert_eq!(mapper.status().mapped_pages(), 2);
    Ok(())
}

#[test]
fn mapper_mapped_range_check_rejects_gaps_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_mapped_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_mapped_range_check_rejects_invalid_ranges() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;
    let max_pages = (4 * PAGE_TABLE_ENTRIES) as u64;

    assert_eq!(
        mapper.ensure_mapped_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_mapped_contiguous(KERNEL_VIRT, max_pages + 1),
        Err(PageTableError::RangeTooLarge)
    );
    assert_eq!(
        mapper.ensure_mapped_contiguous(VirtAddr::new(0xffff_ffff_ffff_f000), 2),
        Err(PageTableError::AddressOverflow)
    );
    Ok(())
}

#[test]
fn mapper_mapped_range_check_rejects_corrupt_intermediate_leaf() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;

    assert_eq!(
        mapper.ensure_mapped_contiguous(VirtAddr::new(0), 1),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}
