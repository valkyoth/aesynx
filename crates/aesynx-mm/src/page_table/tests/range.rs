use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageTableError, PageTableMapper, TlbFlush};

#[test]
fn mapper_maps_contiguous_range_atomically() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);

    let outcome = mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 3, flags)?;

    assert_eq!(outcome.pages(), 3);
    assert_eq!(outcome.flush(), TlbFlush::AddressSpace);
    assert_eq!(mapper.status().mapped_pages, 3);
    assert_eq!(mapper.translate(KERNEL_VIRT), Some(KERNEL_PHYS));
    assert_eq!(
        mapper.translate(VirtAddr::new(KERNEL_VIRT.get() + 0x2000)),
        Some(PhysAddr::new(KERNEL_PHYS.get() + 0x2000))
    );
    Ok(())
}

#[test]
fn mapper_contiguous_map_failure_is_atomic() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + 0x1000),
        KERNEL_PHYS,
        flags,
    )?;
    let before = mapper;

    assert_eq!(
        mapper.map_contiguous(
            KERNEL_VIRT,
            PhysAddr::new(KERNEL_PHYS.get() + 0x4000),
            2,
            flags,
        ),
        Err(PageTableError::AlreadyMapped)
    );
    assert_eq!(mapper, before);
    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Err(PageTableError::NotMapped)
    );
    Ok(())
}

#[test]
fn mapper_contiguous_ranges_reject_malformed_virtual_ranges() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let before = mapper;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);

    assert_eq!(
        mapper.map_contiguous(VirtAddr::new(0x0000_7fff_ffff_f000), KERNEL_PHYS, 2, flags),
        Err(PageTableError::InvalidVirtualAddress)
    );
    assert_eq!(
        mapper.ensure_unmapped_contiguous(VirtAddr::new(0xffff_ffff_ffff_f000), 2),
        Err(PageTableError::AddressOverflow)
    );
    assert_eq!(
        mapper.protect_contiguous(KERNEL_VIRT, u64::MAX, flags),
        Err(PageTableError::AddressOverflow)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_contiguous_map_rejects_malformed_physical_ranges() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let before = mapper;

    assert_eq!(
        mapper.map_contiguous(
            KERNEL_VIRT,
            PhysAddr::new(0x000f_ffff_ffff_f000),
            2,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ),
        Err(PageTableError::InvalidPhysicalAddress)
    );
    assert_eq!(
        mapper.map_contiguous(
            KERNEL_VIRT,
            KERNEL_PHYS,
            u64::MAX,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ),
        Err(PageTableError::AddressOverflow)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_reports_contiguous_range_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 3, flags)?;
    let before = mapper;

    let mapping = mapper.mapping_for_contiguous(KERNEL_VIRT, 3)?;

    assert_eq!(mapping.start_phys(), KERNEL_PHYS);
    assert_eq!(mapping.pages(), 3);
    assert_eq!(mapping.flags(), flags);
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_verifies_unmapped_contiguous_range_without_mutation() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;
    let before = mapper;

    mapper.ensure_unmapped_contiguous(KERNEL_VIRT, 3)?;

    assert_eq!(mapper, before);
    assert_eq!(mapper.status().mapped_pages, 0);
    Ok(())
}

#[test]
fn mapper_unmapped_range_check_rejects_any_mapped_page() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let second = VirtAddr::new(KERNEL_VIRT.get() + 0x1000);
    mapper.map_page(
        second,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.ensure_unmapped_contiguous(KERNEL_VIRT, 3),
        Err(PageTableError::AlreadyMapped)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_contiguous_range_lookup_rejects_gaps_or_mismatches() -> Result<(), PageTableError> {
    let mut gap = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    gap.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    assert_eq!(
        gap.mapping_for_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NotMapped)
    );

    let mut mismatched_phys = PageTableMapper::<4>::new()?;
    mismatched_phys.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mismatched_phys.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + 0x1000),
        PhysAddr::new(KERNEL_PHYS.get() + 0x3000),
        flags,
    )?;
    assert_eq!(
        mismatched_phys.mapping_for_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NonContiguousRange)
    );

    let mut mismatched_flags = PageTableMapper::<4>::new()?;
    mismatched_flags.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mismatched_flags.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + 0x1000),
        PhysAddr::new(KERNEL_PHYS.get() + 0x1000),
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;
    assert_eq!(
        mismatched_flags.mapping_for_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::NonContiguousRange)
    );
    Ok(())
}

#[test]
fn mapper_protects_contiguous_range_atomically() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let initial = GenericPageFlags::kernel(PageAccess::ReadWrite);
    let protected = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, initial)?;

    let outcome = mapper.protect_contiguous(KERNEL_VIRT, 2, protected)?;

    assert_eq!(outcome.pages(), 2);
    assert_eq!(outcome.flush(), TlbFlush::AddressSpace);
    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Ok(crate::page_table::PageMapping::new(KERNEL_PHYS, protected))
    );
    assert_eq!(
        mapper.mapping_for_page(VirtAddr::new(KERNEL_VIRT.get() + 0x1000)),
        Ok(crate::page_table::PageMapping::new(
            PhysAddr::new(KERNEL_PHYS.get() + 0x1000),
            protected
        ))
    );
    assert_eq!(mapper.status().mapped_pages, 2);
    Ok(())
}

#[test]
fn mapper_contiguous_protect_failure_is_atomic() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let initial = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, initial)?;
    let before = mapper;

    assert_eq!(
        mapper.protect_contiguous(
            KERNEL_VIRT,
            3,
            GenericPageFlags::kernel(PageAccess::ReadWrite)
        ),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(mapper, before);
    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Ok(crate::page_table::PageMapping::new(KERNEL_PHYS, initial))
    );
    Ok(())
}

#[test]
fn mapper_unmaps_contiguous_range_atomically() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, flags)?;

    let outcome = mapper.unmap_contiguous(KERNEL_VIRT, 2)?;

    assert_eq!(outcome.pages(), 2);
    assert_eq!(outcome.flush(), TlbFlush::AddressSpace);
    assert_eq!(mapper.status().mapped_pages, 0);
    assert_eq!(mapper.status().used_tables, 1);
    assert_eq!(mapper.translate(KERNEL_VIRT), None);
    Ok(())
}

#[test]
fn mapper_contiguous_unmap_failure_is_atomic() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, flags)?;
    let before = mapper;

    assert_eq!(
        mapper.unmap_contiguous(KERNEL_VIRT, 3),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(mapper, before);
    assert_eq!(mapper.status().mapped_pages, 2);
    assert_eq!(mapper.translate(KERNEL_VIRT), Some(KERNEL_PHYS));
    Ok(())
}

#[test]
fn mapper_contiguous_ranges_reject_zero_pages() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;

    assert_eq!(
        mapper.map_contiguous(
            KERNEL_VIRT,
            KERNEL_PHYS,
            0,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.unmap_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.protect_contiguous(
            KERNEL_VIRT,
            0,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.mapping_for_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(
        mapper.ensure_unmapped_contiguous(KERNEL_VIRT, 0),
        Err(PageTableError::InvalidPageCount)
    );
    assert_eq!(mapper.status().mapped_pages, 0);
    Ok(())
}
