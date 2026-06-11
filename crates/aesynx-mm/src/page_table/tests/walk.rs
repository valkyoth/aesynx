use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{
    PageMapping, PageTableError, PageTableMapper, PageTableMapping, PageTableSlot,
};

#[test]
fn mapper_visits_no_mappings_in_empty_arena() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<4>::new()?;
    let mut called = false;

    let visited = mapper.visit_mappings(|_entry| {
        called = true;
        Ok(())
    })?;

    assert_eq!(visited, 0);
    assert!(!called);
    Ok(())
}

#[test]
fn mapper_visits_leaf_mappings_in_virtual_order() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let second_virt = aesynx_abi::VirtAddr::new(KERNEL_VIRT.get() + crate::FRAME_SIZE);
    let second_phys = aesynx_abi::PhysAddr::new(KERNEL_PHYS.get() + crate::FRAME_SIZE);
    mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, flags)?;
    mapper.map_page(second_virt, second_phys, flags)?;

    let mut index = 0usize;
    let mut seen = [None; 2];
    let before = mapper;
    let visited = mapper.visit_mappings(|entry| {
        if index >= seen.len() {
            return Err(PageTableError::CorruptTable);
        }
        seen[index] = Some(entry);
        index += 1;
        Ok(())
    })?;

    assert_eq!(visited, 2);
    assert_eq!(index, 2);
    assert_eq!(
        seen[0],
        Some(PageTableMapping::new(
            KERNEL_VIRT,
            PageMapping::new(KERNEL_PHYS, flags)
        ))
    );
    assert_eq!(
        seen[1],
        Some(PageTableMapping::new(
            second_virt,
            PageMapping::new(second_phys, flags)
        ))
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_mapping_visitor_propagates_callback_errors() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    assert_eq!(
        mapper.visit_mappings(|_entry| Err(PageTableError::InvalidPageCount)),
        Err(PageTableError::InvalidPageCount)
    );
    Ok(())
}

#[test]
fn mapper_mapping_visitor_rejects_accounting_drift() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 0;
    let mut called = false;

    assert_eq!(
        mapper.visit_mappings(|_entry| {
            called = true;
            Ok(())
        }),
        Err(PageTableError::CorruptTable)
    );
    assert!(!called);
    Ok(())
}

#[test]
fn mapper_mapping_visitor_rejects_unreachable_used_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[1] = true;

    assert_eq!(
        mapper.visit_mappings(|_entry| Ok(())),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    Ok(())
}

#[test]
fn mapper_mapping_visitor_rejects_nonempty_unused_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.tables[1].slots[0] = PageTableSlot::leaf(PageMapping::new(
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    ))?;

    assert_eq!(
        mapper.visit_mappings(|_entry| Ok(())),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    Ok(())
}

#[test]
fn mapper_mapping_visitor_rejects_intermediate_leaf_slots() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let mapping = PageMapping::new(KERNEL_PHYS, GenericPageFlags::kernel(PageAccess::ReadOnly));
    mapper.tables[0].slots[0] = PageTableSlot::leaf(mapping)?;
    mapper.mapped_pages = 1;

    assert_eq!(
        mapper.visit_mappings(|_entry| Ok(())),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    Ok(())
}

#[test]
fn mapper_mapping_visitor_rejects_table_cycles_without_leaf_mappings() -> Result<(), PageTableError>
{
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.tables[0].slots[0] = PageTableSlot { raw: 1 | (1 << 9) };

    assert_eq!(
        mapper.visit_mappings(|_entry| Ok(())),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    Ok(())
}

#[test]
fn mapper_mapping_visitor_rejects_duplicate_empty_child_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.used[1] = true;
    mapper.tables[0].slots[0] = PageTableSlot::next(1)?;
    mapper.tables[0].slots[1] = PageTableSlot::next(1)?;

    assert_eq!(
        mapper.visit_mappings(|_entry| Ok(())),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper.audit(), Err(PageTableError::CorruptTable));
    Ok(())
}
