use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageMapping, PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_summarizes_mapping_classes_without_addresses() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    let global = GenericPageFlags::kernel(PageAccess::ReadOnly)
        .with_global()
        .map_err(|_error| PageTableError::InvalidMappingFlags)?;
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, global)?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE),
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE * 2),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE * 2),
        GenericPageFlags::user(PageAccess::ReadExecute),
    )?;
    mapper.map_page(
        VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE * 3),
        PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE * 3),
        GenericPageFlags::kernel(PageAccess::ReadOnly).device(),
    )?;
    let before = mapper;

    let summary = mapper.mapping_summary()?;
    assert_eq!(summary.total_pages(), 4);
    assert_eq!(summary.kernel_pages(), 3);
    assert_eq!(summary.user_pages(), 1);
    assert_eq!(summary.writable_pages(), 1);
    assert_eq!(summary.executable_pages(), 1);
    assert_eq!(summary.global_pages(), 1);
    assert_eq!(summary.device_pages(), 1);
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_summary_reports_empty_arena() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;

    let summary = mapper.mapping_summary()?;
    assert_eq!(summary.total_pages(), 0);
    assert_eq!(summary.kernel_pages(), 0);
    assert_eq!(summary.user_pages(), 0);
    assert_eq!(summary.writable_pages(), 0);
    assert_eq!(summary.executable_pages(), 0);
    assert_eq!(summary.global_pages(), 0);
    assert_eq!(summary.device_pages(), 0);
    Ok(())
}

#[test]
fn mapper_summary_rejects_corrupt_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;
    let corrupt = mapper;

    assert_eq!(mapper.mapping_summary(), Err(PageTableError::CorruptTable));
    assert_eq!(mapper, corrupt);
    Ok(())
}

#[test]
fn mapper_summary_rejects_accounting_drift_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 0;
    let corrupt = mapper;

    assert_eq!(mapper.mapping_summary(), Err(PageTableError::CorruptTable));
    assert_eq!(mapper, corrupt);
    Ok(())
}
