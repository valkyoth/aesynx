use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::address::page_indices;
use crate::page_table::{PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_map_page_rejects_accounting_drift_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mapper.mapped_pages = 0;
    let corrupt = mapper;

    assert_eq!(
        mapper.map_page(
            VirtAddr::new(KERNEL_VIRT.get() + 0x1000),
            PhysAddr::new(KERNEL_PHYS.get() + 0x1000),
            flags,
        ),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_protect_page_rejects_accounting_drift_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let initial = GenericPageFlags::kernel(PageAccess::ReadWrite);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, initial)?;
    mapper.mapped_pages = 2;
    let corrupt = mapper;

    assert_eq!(
        mapper.protect_page(KERNEL_VIRT, GenericPageFlags::kernel(PageAccess::ReadOnly)),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_unmap_page_rejects_accounting_drift_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mapper.mapped_pages = 2;
    let corrupt = mapper;

    assert_eq!(
        mapper.unmap_page(KERNEL_VIRT),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_map_range_rejects_accounting_drift_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mapper.mapped_pages = 0;
    let corrupt = mapper;

    assert_eq!(
        mapper.map_contiguous(
            VirtAddr::new(KERNEL_VIRT.get() + 0x2000),
            PhysAddr::new(KERNEL_PHYS.get() + 0x2000),
            2,
            flags,
        ),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_protect_range_rejects_accounting_drift_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let initial = GenericPageFlags::kernel(PageAccess::ReadWrite);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, initial)?;
    mapper.mapped_pages = 3;
    let corrupt = mapper;

    assert_eq!(
        mapper.protect_contiguous(
            KERNEL_VIRT,
            2,
            GenericPageFlags::kernel(PageAccess::ReadOnly)
        ),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_unmap_range_rejects_accounting_drift_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_contiguous(
        KERNEL_VIRT,
        KERNEL_PHYS,
        2,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 3;
    let corrupt = mapper;

    assert_eq!(
        mapper.unmap_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_map_page_rejects_malformed_next_link_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let indices = page_indices(KERNEL_VIRT);
    mapper.tables[0].slots[indices[0]] = PageTableSlot { raw: 1 << 9 };
    let corrupt = mapper;

    assert_eq!(
        mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_protect_page_rejects_malformed_next_link_without_mutation() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    let indices = page_indices(KERNEL_VIRT);
    mapper.tables[0].slots[indices[0]] = PageTableSlot { raw: 1 << 9 };
    let corrupt = mapper;

    assert_eq!(
        mapper.protect_page(KERNEL_VIRT, flags),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_unmap_page_rejects_malformed_next_link_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let indices = page_indices(KERNEL_VIRT);
    mapper.tables[0].slots[indices[0]] = PageTableSlot { raw: 1 << 9 };
    let corrupt = mapper;

    assert_eq!(
        mapper.unmap_page(KERNEL_VIRT),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_map_range_rejects_malformed_next_link_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let indices = page_indices(KERNEL_VIRT);
    mapper.tables[0].slots[indices[0]] = PageTableSlot { raw: 1 << 9 };
    let corrupt = mapper;

    assert_eq!(
        mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, flags),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_protect_range_rejects_malformed_next_link_without_mutation() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_contiguous(
        KERNEL_VIRT,
        KERNEL_PHYS,
        2,
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;
    let indices = page_indices(KERNEL_VIRT);
    mapper.tables[0].slots[indices[0]] = PageTableSlot { raw: 1 << 9 };
    let corrupt = mapper;

    assert_eq!(
        mapper.protect_contiguous(
            KERNEL_VIRT,
            2,
            GenericPageFlags::kernel(PageAccess::ReadOnly)
        ),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_unmap_range_rejects_malformed_next_link_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_contiguous(
        KERNEL_VIRT,
        KERNEL_PHYS,
        2,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let indices = page_indices(KERNEL_VIRT);
    mapper.tables[0].slots[indices[0]] = PageTableSlot { raw: 1 << 9 };
    let corrupt = mapper;

    assert_eq!(
        mapper.unmap_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}
