use aesynx_abi::VirtAddr;

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageTableError, PageTableMapper};

const USER_VIRT: VirtAddr = VirtAddr::new(0x0000_0000_0040_0000);

#[test]
fn mapper_read_only_range_checks_reject_accounting_drift_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, flags)?;
    mapper.mapped_pages = 3;
    let corrupt = mapper.clone();

    assert_eq!(
        mapper.mapping_for_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    assert_eq!(
        mapper.ensure_mapped_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    assert_eq!(
        mapper.ensure_unmapped_contiguous(VirtAddr::new(KERNEL_VIRT.get() + 0x4000), 1),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    assert_eq!(
        mapper.ensure_contiguous_flags(KERNEL_VIRT, 2, flags),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_range_policy_checks_reject_accounting_drift_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_contiguous(
        KERNEL_VIRT,
        KERNEL_PHYS,
        2,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 3;
    let corrupt = mapper.clone();

    assert_eq!(
        mapper.ensure_kernel_mapped_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    assert_eq!(
        mapper.ensure_user_mapped_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    assert_eq!(
        mapper.ensure_write_protected_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    assert_eq!(
        mapper.ensure_non_executable_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    assert_eq!(
        mapper.ensure_normal_memory_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    assert_eq!(
        mapper.ensure_local_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    assert_eq!(
        mapper.ensure_kernel_space_contiguous(KERNEL_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_user_space_range_check_rejects_accounting_drift_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_contiguous(
        USER_VIRT,
        KERNEL_PHYS,
        2,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 3;
    let corrupt = mapper.clone();

    assert_eq!(
        mapper.ensure_user_space_contiguous(USER_VIRT, 2),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, corrupt);
    Ok(())
}
