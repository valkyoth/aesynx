use aesynx_abi::{PhysAddr, VirtAddr};

use crate::page_table::address::page_indices;
use crate::page_table::{PageTableError, PageTableMapper, PageTableSlot, TlbFlush};
use crate::{FRAME_SIZE, GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};

const PROPERTY_PAGES: u64 = 24;

#[test]
fn mapper_property_map_unmap_round_trip_restores_empty_state() -> Result<(), PageTableError> {
    let empty = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);

    let mut offset = 0u64;
    while offset < PROPERTY_PAGES {
        let mut mapper = PageTableMapper::<8>::new()?;
        let virt = add_pages_to_virt(KERNEL_VIRT, offset);
        let phys = add_pages_to_phys(KERNEL_PHYS, offset);

        let map = mapper.map_page(virt, phys, flags)?;
        assert_eq!(map.flush(), TlbFlush::Page(virt));
        assert_eq!(mapper.translate(virt), Ok(phys));
        assert_eq!(
            mapper.translate(VirtAddr::new(virt.get() + 0x123)),
            Ok(PhysAddr::new(phys.get() + 0x123))
        );
        assert_eq!(mapper.audit()?.mapped_pages(), 1);

        let unmap = mapper.unmap_page(virt)?;
        assert_eq!(unmap.mapping().phys(), phys);
        assert_eq!(unmap.flush(), TlbFlush::Page(virt));
        assert_eq!(mapper, empty);

        offset += 1;
    }

    Ok(())
}

#[test]
fn mapper_property_failed_single_page_operations_are_atomic() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;

    let mut offset = 0u64;
    while offset < PROPERTY_PAGES {
        let before = mapper;
        let virt = add_pages_to_virt(KERNEL_VIRT, offset);
        let phys = add_pages_to_phys(KERNEL_PHYS, offset + 1);

        let result = mapper.map_page(virt, phys, flags);
        if virt == KERNEL_VIRT {
            assert_eq!(result, Err(PageTableError::AlreadyMapped));
            assert_eq!(mapper, before);
        } else {
            assert!(result.is_ok());
            assert_eq!(
                mapper.audit()?.mapped_pages(),
                before.audit()?.mapped_pages() + 1
            );
            mapper.unmap_page(virt)?;
        }

        let before = mapper;
        assert_eq!(
            mapper.unmap_page(add_pages_to_virt(KERNEL_VIRT, PROPERTY_PAGES + offset)),
            Err(PageTableError::NotMapped)
        );
        assert_eq!(mapper, before);

        offset += 1;
    }

    Ok(())
}

#[test]
fn mapper_property_duplicate_physical_frames_are_rejected_without_mutation()
-> Result<(), PageTableError> {
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);

    let mut offset = 1u64;
    while offset < PROPERTY_PAGES {
        let mut mapper = PageTableMapper::<8>::new()?;
        mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
        let before = mapper;

        assert_eq!(
            mapper.map_page(add_pages_to_virt(KERNEL_VIRT, offset), KERNEL_PHYS, flags),
            Err(PageTableError::PhysicalAlias)
        );
        assert_eq!(mapper, before);

        offset += 1;
    }

    Ok(())
}

#[test]
fn mapper_property_contiguous_range_round_trips_and_walk_bounds() -> Result<(), PageTableError> {
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);

    let mut pages = 1u64;
    while pages <= PROPERTY_PAGES {
        let mut mapper = PageTableMapper::<8, 32>::new()?;
        let before = mapper;

        assert_eq!(
            mapper.ensure_unmapped_contiguous(KERNEL_VIRT, pages),
            Ok(())
        );
        let mapped = mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, pages, flags)?;
        assert_eq!(mapped.pages(), pages);
        assert_eq!(mapper.ensure_mapped_contiguous(KERNEL_VIRT, pages), Ok(()));
        assert_eq!(
            mapper
                .mapping_for_contiguous(KERNEL_VIRT, pages)?
                .start_phys(),
            KERNEL_PHYS
        );
        assert_eq!(mapper.audit()?.mapped_pages(), pages);

        let unmapped = mapper.unmap_contiguous(KERNEL_VIRT, pages)?;
        assert_eq!(unmapped.pages(), pages);
        assert_eq!(mapper, before);

        pages += 1;
    }

    let mapper = PageTableMapper::<8, 32>::new()?;
    assert_eq!(
        mapper.ensure_unmapped_contiguous(KERNEL_VIRT, 33),
        Err(PageTableError::RangeTooLarge)
    );

    Ok(())
}

#[test]
fn mapper_property_audit_detects_table_index_and_accounting_drift() -> Result<(), PageTableError> {
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;

    let mut accounting_drift = mapper;
    accounting_drift.mapped_pages = 0;
    assert_eq!(accounting_drift.audit(), Err(PageTableError::CorruptTable));

    let mut stale_index = mapper;
    let indices = page_indices(KERNEL_VIRT);
    stale_index.tables[0].slots[indices[0]] = PageTableSlot { raw: 0 };
    assert_eq!(stale_index.audit(), Err(PageTableError::CorruptTable));

    let mut malformed_next = mapper;
    malformed_next.tables[0].slots[indices[0]] = PageTableSlot { raw: 1 << 9 };
    assert_eq!(malformed_next.audit(), Err(PageTableError::CorruptTable));

    Ok(())
}

fn add_pages_to_virt(base: VirtAddr, pages: u64) -> VirtAddr {
    VirtAddr::new(base.get() + pages * FRAME_SIZE)
}

fn add_pages_to_phys(base: PhysAddr, pages: u64) -> PhysAddr {
    PhysAddr::new(base.get() + pages * FRAME_SIZE)
}
