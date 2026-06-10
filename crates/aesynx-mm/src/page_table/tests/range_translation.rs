use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{FRAME_SIZE, GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PAGE_TABLE_ENTRIES, PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_translates_byte_range_within_single_page() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;

    let translated = mapper
        .translate_contiguous_range_checked(VirtAddr::new(KERNEL_VIRT.get() + 0x123), 0x80)?;

    assert_eq!(
        translated.start_phys(),
        PhysAddr::new(KERNEL_PHYS.get() + 0x123)
    );
    assert_eq!(translated.byte_len(), 0x80);
    assert_eq!(translated.pages(), 1);
    assert_eq!(translated.flags(), flags);
    Ok(())
}

#[test]
fn mapper_translates_byte_range_across_contiguous_pages() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadWrite);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 4, flags)?;

    let translated = mapper
        .translate_contiguous_range_checked(VirtAddr::new(KERNEL_VIRT.get() + 0xff0), 0x2020)?;

    assert_eq!(
        translated.start_phys(),
        PhysAddr::new(KERNEL_PHYS.get() + 0xff0)
    );
    assert_eq!(translated.byte_len(), 0x2020);
    assert_eq!(translated.pages(), 4);
    assert_eq!(translated.flags(), flags);
    Ok(())
}

#[test]
fn mapper_translate_byte_range_rejects_malformed_ranges() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(
        mapper.translate_contiguous_range_checked(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidByteCount)
    );
    assert_eq!(
        mapper.translate_contiguous_range_checked(VirtAddr::new(0x0000_8000_0000_0000), 1),
        Err(PageTableError::InvalidVirtualAddress)
    );
    assert_eq!(
        mapper.translate_contiguous_range_checked(VirtAddr::new(0x0000_7fff_ffff_ffff), 2),
        Err(PageTableError::InvalidVirtualAddress)
    );
    assert_eq!(
        mapper.translate_contiguous_range_checked(VirtAddr::new(u64::MAX), 2),
        Err(PageTableError::AddressOverflow)
    );
    Ok(())
}

#[test]
fn mapper_translate_byte_range_rejects_gaps_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.translate_contiguous_range_checked(VirtAddr::new(KERNEL_VIRT.get() + 0xff0), 0x20),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_translate_byte_range_rejects_noncontiguous_physical_pages() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + 0x3000),
        flags,
    )?;
    let before = mapper;

    assert_eq!(
        mapper.translate_contiguous_range_checked(VirtAddr::new(KERNEL_VIRT.get() + 0xff0), 0x20),
        Err(PageTableError::NonContiguousRange)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_translate_byte_range_rejects_flag_mismatch() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + FRAME_SIZE),
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.translate_contiguous_range_checked(VirtAddr::new(KERNEL_VIRT.get() + 0xff0), 0x20),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_translate_byte_range_rejects_corrupt_leaf() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_contiguous(
        KERNEL_VIRT,
        KERNEL_PHYS,
        2,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.tables[3].slots[1] = PageTableSlot {
        raw: (KERNEL_PHYS.get() + FRAME_SIZE) | 1 | (1 << 1),
    };

    assert_eq!(
        mapper.translate_contiguous_range_checked(VirtAddr::new(KERNEL_VIRT.get() + 0xff0), 0x20),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}

#[test]
fn mapper_translate_byte_range_is_walk_bounded() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;
    let max_pages = (4 * PAGE_TABLE_ENTRIES) as u64;
    let too_many_bytes = max_pages
        .checked_mul(FRAME_SIZE)
        .and_then(|value| value.checked_add(1))
        .ok_or(PageTableError::AddressOverflow)?;

    assert_eq!(
        mapper.translate_contiguous_range_checked(KERNEL_VIRT, too_many_bytes),
        Err(PageTableError::RangeTooLarge)
    );
    Ok(())
}
