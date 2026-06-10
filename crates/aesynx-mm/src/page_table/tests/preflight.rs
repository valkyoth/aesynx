use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PAGE_TABLE_LEVELS, PageTableError, PageTableMapper, PageTableSlot};

const USER_VIRT: VirtAddr = VirtAddr::new(0x0000_0000_0040_0000);

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
fn mapper_kernel_candidate_preflight_rejects_empty_address_space() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_kernel_candidate_rejects_without_mutation(&mapper, PageTableError::EmptyAddressSpace);
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

    assert_kernel_candidate_rejects_without_mutation(
        &mapper,
        PageTableError::UnexpectedMappingFlags,
    );
    Ok(())
}

#[test]
fn mapper_kernel_candidate_preflight_rejects_low_half_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        VirtAddr::new(0x0000_0000_0040_0000),
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    assert_kernel_candidate_rejects_without_mutation(
        &mapper,
        PageTableError::UnexpectedVirtualAddressSpace,
    );
    Ok(())
}

#[test]
fn mapper_kernel_candidate_preflight_reports_low_half_before_user_flags()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        USER_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;

    assert_kernel_candidate_rejects_without_mutation(
        &mapper,
        PageTableError::UnexpectedVirtualAddressSpace,
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

    assert_kernel_candidate_rejects_without_mutation(&mapper, PageTableError::PhysicalAlias);
    Ok(())
}

#[test]
fn mapper_kernel_candidate_preflight_rejects_device_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly).device(),
    )?;

    assert_kernel_candidate_rejects_without_mutation(
        &mapper,
        PageTableError::UnexpectedMappingFlags,
    );
    Ok(())
}

#[test]
fn mapper_kernel_candidate_preflight_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[1] = true;
    mapper.tables[0].slots[0] = PageTableSlot::next(1)?;
    mapper.tables[0].slots[1] = PageTableSlot::next(1)?;

    assert_kernel_candidate_rejects_without_mutation(&mapper, PageTableError::CorruptTable);
    Ok(())
}

#[test]
fn mapper_kernel_candidate_preflight_reports_corruption_before_policy() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages += 1;

    assert_kernel_candidate_rejects_without_mutation(&mapper, PageTableError::CorruptTable);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_accepts_split_address_space_without_mutation()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        USER_VIRT,
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    let before = mapper;

    let audit = mapper.verify_user_address_space_candidate()?;

    assert_eq!(audit.total_tables(), 8);
    assert_eq!(audit.used_tables(), (PAGE_TABLE_LEVELS * 2 - 1) as u64);
    assert_eq!(audit.reachable_tables(), (PAGE_TABLE_LEVELS * 2 - 1) as u64);
    assert_eq!(audit.mapped_pages(), 2);
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_rejects_empty_address_space() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    assert_user_candidate_rejects_without_mutation(&mapper, PageTableError::EmptyAddressSpace);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_rejects_kernel_only_address_space() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    assert_user_candidate_rejects_without_mutation(&mapper, PageTableError::IncompleteAddressSpace);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_reports_incomplete_before_global_policy()
-> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let global = GenericPageFlags::kernel(PageAccess::ReadOnly)
        .with_global()
        .map_err(|_error| PageTableError::InvalidMappingFlags)?;
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, global)?;

    assert_user_candidate_rejects_without_mutation(&mapper, PageTableError::IncompleteAddressSpace);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_rejects_high_half_user_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;

    assert_user_candidate_rejects_without_mutation(&mapper, PageTableError::UnexpectedMappingFlags);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_rejects_low_half_kernel_mappings() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        USER_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    assert_user_candidate_rejects_without_mutation(&mapper, PageTableError::UnexpectedMappingFlags);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_rejects_physical_aliases() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.map_page(
        USER_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;

    assert_user_candidate_rejects_without_mutation(&mapper, PageTableError::PhysicalAlias);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_rejects_device_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        USER_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly).device(),
    )?;

    assert_user_candidate_rejects_without_mutation(&mapper, PageTableError::UnexpectedMappingFlags);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_rejects_global_mappings() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        USER_VIRT,
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;
    let global = GenericPageFlags::kernel(PageAccess::ReadOnly)
        .with_global()
        .map_err(|_error| PageTableError::InvalidMappingFlags)?;
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, global)?;

    assert_user_candidate_rejects_without_mutation(&mapper, PageTableError::UnexpectedMappingFlags);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[1] = true;
    mapper.tables[0].slots[0] = PageTableSlot::next(1)?;
    mapper.tables[0].slots[1] = PageTableSlot::next(1)?;

    assert_user_candidate_rejects_without_mutation(&mapper, PageTableError::CorruptTable);
    Ok(())
}

#[test]
fn mapper_user_candidate_preflight_reports_corruption_before_policy() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        USER_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages += 1;

    assert_user_candidate_rejects_without_mutation(&mapper, PageTableError::CorruptTable);
    Ok(())
}

fn assert_kernel_candidate_rejects_without_mutation<const TABLES: usize>(
    mapper: &PageTableMapper<TABLES>,
    error: PageTableError,
) {
    let before = *mapper;

    assert_eq!(mapper.verify_kernel_address_space_candidate(), Err(error));
    assert_eq!(*mapper, before);
}

fn assert_user_candidate_rejects_without_mutation<const TABLES: usize>(
    mapper: &PageTableMapper<TABLES>,
    error: PageTableError,
) {
    let before = *mapper;

    assert_eq!(mapper.verify_user_address_space_candidate(), Err(error));
    assert_eq!(*mapper, before);
}
