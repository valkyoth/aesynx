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
    assert_eq!(mapper.status().mapped_pages, 0);
    Ok(())
}
