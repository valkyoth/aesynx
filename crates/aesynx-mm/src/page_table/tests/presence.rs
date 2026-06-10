use aesynx_abi::VirtAddr;

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageMapping, PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_reports_page_presence_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;

    assert_eq!(mapper.is_page_mapped(KERNEL_VIRT), Ok(false));

    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    assert_eq!(mapper.is_page_mapped(KERNEL_VIRT), Ok(true));
    assert_eq!(mapper, before);
    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Ok(PageMapping::new(
            KERNEL_PHYS,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ))
    );

    mapper.unmap_page(KERNEL_VIRT)?;

    assert_eq!(mapper.is_page_mapped(KERNEL_VIRT), Ok(false));
    Ok(())
}

#[test]
fn mapper_page_presence_rejects_invalid_virtual_addresses() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_eq!(
        mapper.is_page_mapped(VirtAddr::new(0x0000_8000_0000_0000)),
        Err(PageTableError::InvalidVirtualAddress)
    );
    assert_eq!(
        mapper.is_page_mapped(VirtAddr::new(KERNEL_VIRT.get() + 1)),
        Err(PageTableError::UnalignedVirtualAddress)
    );
    Ok(())
}

#[test]
fn mapper_page_presence_rejects_corrupt_intermediate_leaf() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;

    assert_eq!(
        mapper.is_page_mapped(VirtAddr::new(0)),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}
