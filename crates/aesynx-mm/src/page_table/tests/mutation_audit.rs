use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageTableError, PageTableMapper};

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
