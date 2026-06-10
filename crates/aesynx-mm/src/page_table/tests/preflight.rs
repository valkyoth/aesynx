use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PAGE_TABLE_LEVELS, PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_kernel_candidate_preflight_accepts_kernel_mappings_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;
    let before = mapper;

    let audit = mapper.verify_kernel_address_space_candidate()?;

    assert_eq!(audit.total_tables(), 8);
    assert_eq!(audit.used_tables(), PAGE_TABLE_LEVELS as u64);
    assert_eq!(audit.reachable_tables(), PAGE_TABLE_LEVELS as u64);
    assert_eq!(audit.mapped_pages(), 2);
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_kernel_candidate_preflight_rejects_user_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;

    assert_eq!(
        mapper.verify_kernel_address_space_candidate(),
        Err(PageTableError::UnexpectedMappingFlags)
    );
    Ok(())
}

#[test]
fn mapper_kernel_candidate_preflight_rejects_physical_aliases() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE),
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    assert_eq!(
        mapper.verify_kernel_address_space_candidate(),
        Err(PageTableError::PhysicalAlias)
    );
    Ok(())
}

#[test]
fn mapper_kernel_candidate_preflight_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[1] = true;
    mapper.tables[0].slots[0] = PageTableSlot::next(1)?;
    mapper.tables[0].slots[1] = PageTableSlot::next(1)?;

    assert_eq!(
        mapper.verify_kernel_address_space_candidate(),
        Err(PageTableError::CorruptTable)
    );
    Ok(())
}
