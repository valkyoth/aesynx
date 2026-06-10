use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess, PagePrivilege};

use super::{
    PAGE_TABLE_LEVELS, PageMapping, PageTableError, PageTableMapper, TlbFlush, X86_64PageTableEntry,
};

const KERNEL_VIRT: VirtAddr = VirtAddr::new(0xffff_9000_0000_0000);
const KERNEL_PHYS: PhysAddr = PhysAddr::new(0x0020_0000);

#[test]
fn mapper_maps_and_translates_page() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadWrite);

    let outcome = mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;

    assert_eq!(outcome.flush(), TlbFlush::Page(KERNEL_VIRT));
    assert_eq!(mapper.translate(KERNEL_VIRT), Some(KERNEL_PHYS));
    assert_eq!(
        mapper.translate(VirtAddr::new(KERNEL_VIRT.get() + 0x123)),
        Some(PhysAddr::new(KERNEL_PHYS.get() + 0x123))
    );
    assert_eq!(mapper.status().mapped_pages, 1);
    assert_eq!(mapper.status().used_tables, PAGE_TABLE_LEVELS as u64);
    Ok(())
}

#[test]
fn mapper_unmaps_page_and_reports_flush() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;

    let outcome = mapper.unmap_page(KERNEL_VIRT)?;

    assert_eq!(outcome.mapping(), PageMapping::new(KERNEL_PHYS, flags));
    assert_eq!(outcome.flush(), TlbFlush::Page(KERNEL_VIRT));
    assert_eq!(mapper.translate(KERNEL_VIRT), None);
    assert_eq!(
        mapper.unmap_page(KERNEL_VIRT),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(mapper.status().mapped_pages, 0);
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
    assert_eq!(mapper.translate(KERNEL_VIRT), Some(KERNEL_PHYS));
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
    assert_eq!(mapper.translate(VirtAddr::new(0x0000_8000_0000_0000)), None);
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
    assert_eq!(mapper.status().used_tables, 1);
    assert_eq!(mapper.status().mapped_pages, 0);
    Ok(())
}

#[test]
fn x86_64_entry_encodes_safe_mapping_flags() -> Result<(), PageTableError> {
    let flags = GenericPageFlags::kernel(PageAccess::ReadExecute)
        .with_global()
        .map_err(|_error| PageTableError::CorruptTable)?;
    let entry = X86_64PageTableEntry::from_mapping(PageMapping::new(KERNEL_PHYS, flags))?;

    assert_eq!(entry.raw() & 1, 1);
    assert_eq!(entry.raw() & (1 << 1), 0);
    assert_eq!(entry.raw() & (1 << 2), 0);
    assert_eq!(entry.raw() & (1 << 8), 1 << 8);
    assert_eq!(entry.raw() & (1 << 63), 0);
    Ok(())
}

#[test]
fn x86_64_entry_encodes_user_nx_device_mapping() -> Result<(), PageTableError> {
    let flags = GenericPageFlags::user(PageAccess::ReadOnly).device();
    let entry = X86_64PageTableEntry::from_mapping(PageMapping::new(KERNEL_PHYS, flags))?;

    assert_eq!(flags.privilege, PagePrivilege::User);
    assert_eq!(entry.raw() & (1 << 2), 1 << 2);
    assert_eq!(entry.raw() & (1 << 3), 1 << 3);
    assert_eq!(entry.raw() & (1 << 4), 1 << 4);
    assert_eq!(entry.raw() & (1 << 63), 1 << 63);
    Ok(())
}
