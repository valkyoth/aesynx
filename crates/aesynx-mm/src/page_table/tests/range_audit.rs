use aesynx_abi::VirtAddr;

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageTableError, PageTableMapper};

#[test]
fn mapper_read_only_range_checks_reject_accounting_drift() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, flags)?;
    mapper.mapped_pages = 3;

    assert_eq!(
        mapper.mapping_for_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.ensure_mapped_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.ensure_unmapped_contiguous(VirtAddr::new(KERNEL_VIRT.get() + 0x4000), 1),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.ensure_contiguous_flags(KERNEL_VIRT, 2, flags),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}

#[test]
fn mapper_range_policy_checks_reject_accounting_drift() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_contiguous(
        KERNEL_VIRT,
        KERNEL_PHYS,
        2,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 3;

    assert_eq!(
        mapper.ensure_kernel_mapped_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.ensure_write_protected_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.ensure_non_executable_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.ensure_normal_memory_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.ensure_local_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.ensure_kernel_space_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}
