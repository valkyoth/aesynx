use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_alias_check_accepts_empty_mapper() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    mapper.ensure_no_physical_aliases()?;

    assert_eq!(mapper.status().mapped_pages(), 0);
    Ok(())
}

#[test]
fn mapper_alias_check_accepts_distinct_frames_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 3, flags)?;
    let before = mapper.clone();

    mapper.ensure_no_physical_aliases()?;

    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_rejects_duplicate_physical_frames_at_map_time() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let alias_virt = VirtAddr::new(KERNEL_VIRT.get() + 0x4000);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    let before = mapper.clone();

    assert_eq!(
        mapper.map_page(alias_virt, KERNEL_PHYS, flags),
        Err(PageTableError::PhysicalAlias)
    );
    assert_eq!(mapper, before);
    assert_eq!(mapper.status().mapped_pages(), 1);
    Ok(())
}

#[test]
fn mapper_rejects_duplicate_physical_frames_with_different_flags() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let alias_virt = VirtAddr::new(KERNEL_VIRT.get() + 0x4000);
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let before = mapper.clone();

    assert_eq!(
        mapper.map_page(
            alias_virt,
            KERNEL_PHYS,
            GenericPageFlags::kernel(PageAccess::ReadExecute),
        ),
        Err(PageTableError::PhysicalAlias)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_rejects_contiguous_ranges_that_alias_existing_frames() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    let before = mapper.clone();

    assert_eq!(
        mapper.map_contiguous(
            VirtAddr::new(KERNEL_VIRT.get() + 0x4000),
            KERNEL_PHYS,
            2,
            flags
        ),
        Err(PageTableError::PhysicalAlias)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_alias_check_trusts_map_time_invariant() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + 0x4000),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        flags,
    )?;

    mapper.ensure_no_physical_aliases()?;
    Ok(())
}

#[test]
fn mapper_alias_check_rederives_alias_invariant_from_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, flags)?;
    mapper.tables[3].slots[1] = PageTableSlot {
        raw: KERNEL_PHYS.get() | 1 | (1 << 63),
    };

    assert_eq!(
        mapper.ensure_no_physical_aliases(),
        Err(PageTableError::PhysicalAlias)
    );
    Ok(())
}

#[test]
fn mapper_alias_check_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.tables[0].slots[0] = PageTableSlot::next(1)?;

    assert_eq!(
        mapper.ensure_no_physical_aliases(),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}

#[test]
fn mapper_alias_check_rejects_accounting_drift() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 2;

    assert_eq!(
        mapper.ensure_no_physical_aliases(),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}

#[test]
fn mapper_alias_check_reports_corruption_before_policy_success() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mapper.mapped_pages = 2;

    assert_eq!(
        mapper.ensure_no_physical_aliases(),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}
