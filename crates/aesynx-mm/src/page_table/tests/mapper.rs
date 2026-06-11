use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{
    PAGE_TABLE_LEVELS, PageMapping, PageTableError, PageTableMapper, TlbFlush,
};

#[test]
fn mapper_rejects_empty_arena() {
    assert_eq!(PageTableMapper::<0>::new(), Err(PageTableError::EmptyArena));
}

#[test]
fn mapper_maps_and_translates_page() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadWrite);

    assert_eq!(mapper.root_table().table_index(), 0);
    let outcome = mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;

    assert_eq!(outcome.flush(), TlbFlush::Page(KERNEL_VIRT));
    assert_eq!(mapper.translate(KERNEL_VIRT), Ok(KERNEL_PHYS));
    assert_eq!(
        mapper.translate(VirtAddr::new(KERNEL_VIRT.get() + 0x123)),
        Ok(PhysAddr::new(KERNEL_PHYS.get() + 0x123))
    );
    assert_eq!(
        mapper.translate_checked(VirtAddr::new(KERNEL_VIRT.get() + 0x123)),
        Ok(PhysAddr::new(KERNEL_PHYS.get() + 0x123))
    );
    assert_eq!(mapper.status().mapped_pages(), 1);
    assert_eq!(mapper.status().used_tables(), PAGE_TABLE_LEVELS as u64);
    assert_eq!(mapper.root_table().table_index(), 0);
    Ok(())
}

#[test]
fn tlb_flush_merge_is_conservative() {
    let first = VirtAddr::new(0xffff_8000_0000_0000);
    let second = VirtAddr::new(0xffff_8000_0000_1000);

    assert_eq!(
        TlbFlush::None.merge(TlbFlush::Page(first)),
        TlbFlush::Page(first)
    );
    assert_eq!(
        TlbFlush::Page(first).merge(TlbFlush::None),
        TlbFlush::Page(first)
    );
    assert_eq!(
        TlbFlush::Page(first).merge(TlbFlush::Page(first)),
        TlbFlush::Page(first)
    );
    assert_eq!(
        TlbFlush::Page(first).merge(TlbFlush::Page(second)),
        TlbFlush::AddressSpace
    );
    assert_eq!(
        TlbFlush::Page(first).merge(TlbFlush::AddressSpace),
        TlbFlush::AddressSpace
    );
}

#[test]
fn mapper_unmaps_page_and_reports_flush() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;

    let outcome = mapper.unmap_page(KERNEL_VIRT)?;

    assert_eq!(outcome.mapping(), PageMapping::new(KERNEL_PHYS, flags));
    assert_eq!(outcome.flush(), TlbFlush::Page(KERNEL_VIRT));
    assert_eq!(
        mapper.translate(KERNEL_VIRT),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        mapper.unmap_page(KERNEL_VIRT),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(mapper.status().mapped_pages(), 0);
    assert_eq!(mapper.status().used_tables(), 1);
    Ok(())
}

#[test]
fn mapper_unmap_global_page_requests_address_space_flush() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly)
        .with_global()
        .map_err(|_error| PageTableError::InvalidMappingFlags)?;
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;

    let outcome = mapper.unmap_page(KERNEL_VIRT)?;

    assert_eq!(outcome.mapping(), PageMapping::new(KERNEL_PHYS, flags));
    assert_eq!(outcome.flush(), TlbFlush::AddressSpace);
    assert_eq!(mapper.status().mapped_pages(), 0);
    Ok(())
}

#[test]
fn mapper_unmap_reclaims_empty_intermediate_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    assert_eq!(mapper.status().used_tables(), PAGE_TABLE_LEVELS as u64);
    mapper.unmap_page(KERNEL_VIRT)?;

    assert_eq!(mapper.status().used_tables(), 1);
    assert_eq!(mapper.status().mapped_pages(), 0);
    Ok(())
}

#[test]
fn mapper_unmap_preserves_tables_needed_by_siblings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let sibling = VirtAddr::new(KERNEL_VIRT.get() + 0x1000);
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        sibling,
        PhysAddr::new(KERNEL_PHYS.get() + 0x1000),
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    mapper.unmap_page(KERNEL_VIRT)?;

    assert_eq!(mapper.status().used_tables(), PAGE_TABLE_LEVELS as u64);
    assert_eq!(mapper.status().mapped_pages(), 1);
    assert_eq!(mapper.translate(sibling), Ok(PhysAddr::new(0x0020_1000)));
    Ok(())
}

#[test]
fn mapper_reports_mapping_flags_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadExecute)
        .with_global()
        .map_err(|_error| PageTableError::InvalidMappingFlags)?;
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    let before = mapper;

    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Ok(PageMapping::new(KERNEL_PHYS, flags))
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_mapping_lookup_rejects_invalid_or_unmapped_pages() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(
        mapper.mapping_for_page(VirtAddr::new(0x0000_8000_0000_0000)),
        Err(PageTableError::InvalidVirtualAddress)
    );
    assert_eq!(
        mapper.mapping_for_page(VirtAddr::new(KERNEL_VIRT.get() + 1)),
        Err(PageTableError::UnalignedVirtualAddress)
    );
    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        mapper.translate_checked(KERNEL_VIRT),
        Err(PageTableError::NotMapped)
    );
    Ok(())
}

#[test]
fn mapper_checked_translation_rejects_accounting_drift() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 2;

    assert_eq!(
        mapper.translate_checked(KERNEL_VIRT),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}

#[test]
fn mapper_protects_existing_page_permissions() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let initial = GenericPageFlags::kernel(PageAccess::ReadWrite);
    let protected = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, initial)?;

    let outcome = mapper.protect_page(KERNEL_VIRT, protected)?;

    assert_eq!(outcome.previous(), PageMapping::new(KERNEL_PHYS, initial));
    assert_eq!(outcome.current(), PageMapping::new(KERNEL_PHYS, protected));
    assert_eq!(outcome.flush(), TlbFlush::Page(KERNEL_VIRT));
    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Ok(PageMapping::new(KERNEL_PHYS, protected))
    );
    assert_eq!(mapper.translate(KERNEL_VIRT), Ok(KERNEL_PHYS));
    assert_eq!(mapper.status().mapped_pages(), 1);
    Ok(())
}

#[test]
fn mapper_protect_global_page_requests_address_space_flush() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let initial = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let global = initial
        .with_global()
        .map_err(|_error| PageTableError::InvalidMappingFlags)?;
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, initial)?;

    let outcome = mapper.protect_page(KERNEL_VIRT, global)?;

    assert_eq!(outcome.previous(), PageMapping::new(KERNEL_PHYS, initial));
    assert_eq!(outcome.current(), PageMapping::new(KERNEL_PHYS, global));
    assert_eq!(outcome.flush(), TlbFlush::AddressSpace);
    Ok(())
}

#[test]
fn mapper_protect_failures_are_atomic() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let initial = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, initial)?;
    let before = mapper;
    let mut invalid = GenericPageFlags::kernel(PageAccess::ReadExecute);
    invalid.device_memory = true;
    invalid.cacheable = false;

    assert_eq!(
        mapper.protect_page(KERNEL_VIRT, invalid),
        Err(PageTableError::InvalidMappingFlags)
    );
    assert_eq!(
        mapper.protect_page(VirtAddr::new(KERNEL_VIRT.get() + 1), initial),
        Err(PageTableError::UnalignedVirtualAddress)
    );
    assert_eq!(
        mapper.protect_page(VirtAddr::new(KERNEL_VIRT.get() + 0x1000), initial),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(mapper, before);
    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Ok(PageMapping::new(KERNEL_PHYS, initial))
    );
    Ok(())
}

#[test]
fn mapper_rejects_double_map_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.map_page(
            KERNEL_VIRT,
            PhysAddr::new(0x0030_0000),
            GenericPageFlags::kernel(PageAccess::ReadWrite),
        ),
        Err(PageTableError::AlreadyMapped)
    );
    assert_eq!(mapper, before);
    assert_eq!(mapper.translate(KERNEL_VIRT), Ok(KERNEL_PHYS));
    Ok(())
}

#[test]
fn mapper_rejects_noncanonical_and_unaligned_addresses() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;

    assert_eq!(
        mapper.map_page(
            VirtAddr::new(0x0000_8000_0000_0000),
            KERNEL_PHYS,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ),
        Err(PageTableError::InvalidVirtualAddress)
    );
    assert_eq!(
        mapper.map_page(
            VirtAddr::new(KERNEL_VIRT.get() + 1),
            KERNEL_PHYS,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ),
        Err(PageTableError::UnalignedVirtualAddress)
    );
    assert_eq!(
        mapper.map_page(
            KERNEL_VIRT,
            PhysAddr::new(KERNEL_PHYS.get() + 1),
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ),
        Err(PageTableError::UnalignedPhysicalAddress)
    );
    assert_eq!(
        mapper.translate(VirtAddr::new(0x0000_8000_0000_0000)),
        Err(PageTableError::InvalidVirtualAddress)
    );
    assert_eq!(
        mapper.translate_checked(VirtAddr::new(0x0000_8000_0000_0000)),
        Err(PageTableError::InvalidVirtualAddress)
    );
    Ok(())
}

#[test]
fn mapper_rejects_physical_address_above_supported_range() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let before = mapper;

    assert_eq!(
        mapper.map_page(
            KERNEL_VIRT,
            PhysAddr::new(0x0010_0000_0000_0000),
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ),
        Err(PageTableError::InvalidPhysicalAddress)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_invalid_mapping_flags_failure_is_atomic() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let before = mapper;
    let mut flags = GenericPageFlags::kernel(PageAccess::ReadExecute);
    flags.device_memory = true;
    flags.cacheable = false;

    assert_eq!(
        mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags),
        Err(PageTableError::InvalidMappingFlags)
    );
    assert_eq!(mapper, before);
    assert_eq!(mapper.status().used_tables(), 1);
    assert_eq!(mapper.status().mapped_pages(), 0);
    Ok(())
}

#[test]
fn mapper_accounting_drift_failure_is_atomic() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.mapped_pages = u64::MAX;
    let before = mapper;

    assert_eq!(
        mapper.map_page(
            KERNEL_VIRT,
            KERNEL_PHYS,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}

#[test]
fn mapper_unmap_validation_failures_are_atomic() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    assert_eq!(
        mapper.unmap_page(VirtAddr::new(0x0000_8000_0000_0000)),
        Err(PageTableError::InvalidVirtualAddress)
    );
    assert_eq!(
        mapper.unmap_page(VirtAddr::new(KERNEL_VIRT.get() + 1)),
        Err(PageTableError::UnalignedVirtualAddress)
    );
    assert_eq!(mapper, before);
    assert_eq!(mapper.translate(KERNEL_VIRT), Ok(KERNEL_PHYS));
    Ok(())
}

#[test]
fn mapper_unmap_rejects_accounting_underflow_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mapper.mapped_pages = 0;
    let before = mapper;

    assert_eq!(
        mapper.unmap_page(KERNEL_VIRT),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}

#[test]
fn mapper_capacity_failure_is_atomic() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<3>::new()?;
    let before = mapper;

    assert_eq!(
        mapper.map_page(
            KERNEL_VIRT,
            KERNEL_PHYS,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ),
        Err(PageTableError::OutOfPageTables)
    );
    assert_eq!(mapper, before);
    assert_eq!(mapper.status().used_tables(), 1);
    assert_eq!(mapper.status().mapped_pages(), 0);
    Ok(())
}

#[test]
fn mapper_capacity_helper_rejects_empty_arena() {
    let mapper = PageTableMapper::<0> {
        tables: [],
        used: [],
        mapped_pages: 0,
    };

    assert_eq!(
        mapper.validate_map_capacity(KERNEL_VIRT),
        Err(PageTableError::EmptyArena)
    );
}
