use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageMapping, PageTableError, PageTableMapper, PageTableSlot};

#[test]
fn mapper_lookup_rejects_corrupt_leaf_slot() -> Result<(), PageTableError> {
    let mapper = mapper_with_corrupt_leaf()?;

    assert_eq!(
        mapper.mapping_for_page(KERNEL_VIRT),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.is_page_mapped(KERNEL_VIRT),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        mapper.translate_checked(KERNEL_VIRT),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper.translate(KERNEL_VIRT), None);
    assert_eq!(mapper.status().mapped_pages, 1);
    Ok(())
}

#[test]
fn mapper_protect_rejects_corrupt_leaf_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_corrupt_leaf()?;
    let before = mapper;

    assert_eq!(
        mapper.protect_page(KERNEL_VIRT, GenericPageFlags::kernel(PageAccess::ReadOnly)),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    Ok(())
}

#[test]
fn mapper_unmap_rejects_corrupt_leaf_without_mutation() -> Result<(), PageTableError> {
    let mut mapper = mapper_with_corrupt_leaf()?;
    let before = mapper;

    assert_eq!(
        mapper.unmap_page(KERNEL_VIRT),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(mapper, before);
    assert_eq!(mapper.status().mapped_pages, 1);
    Ok(())
}

#[test]
fn raw_leaf_decoder_distinguishes_empty_from_corrupt_slots() -> Result<(), PageTableError> {
    let valid = PageTableSlot::leaf(PageMapping::new(
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    ))?;
    let write_execute = PageTableSlot {
        raw: KERNEL_PHYS.get() | 1 | (1 << 1),
    };

    assert_eq!(
        PageTableSlot::EMPTY.leaf_mapping(),
        Err(PageTableError::NotMapped)
    );
    assert_eq!(
        PageTableSlot::next(1)?.leaf_mapping(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        write_execute.leaf_mapping(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        valid.leaf_mapping(),
        Ok(PageMapping::new(
            KERNEL_PHYS,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        ))
    );
    Ok(())
}

fn mapper_with_corrupt_leaf() -> Result<PageTableMapper<4>, PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.tables[3].slots[0] = PageTableSlot {
        raw: KERNEL_PHYS.get() | 1 | (1 << 1),
    };
    Ok(mapper)
}
