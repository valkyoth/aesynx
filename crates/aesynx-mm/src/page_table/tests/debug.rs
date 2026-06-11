use alloc::{format, string::ToString};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{
    PageMapping, PageRangeMapping, PageTableError, PageTableMapper, PageTableMapping, TlbFlush,
    TranslatedRange, X86_64PageTableEntry,
};

#[test]
fn mapper_debug_redacts_page_table_contents() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    let debug = format!("{mapper:?}");

    assert!(debug.contains("PageTableMapper"));
    assert!(debug.contains("total_tables"));
    assert!(debug.contains("used_tables"));
    assert!(debug.contains("mapped_pages"));
    assert!(debug.contains("audit_ok: true"));
    assert!(!debug.contains("slots"));
    assert!(!debug.contains("raw"));
    assert!(!debug.contains(&KERNEL_PHYS.get().to_string()));
    Ok(())
}

#[test]
fn mapper_debug_reports_corruption_without_dumping_tables() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    mapper.mapped_pages = 2;

    let debug = format!("{mapper:?}");

    assert!(debug.contains("audit_ok: false"));
    assert!(!debug.contains("slots"));
    assert!(!debug.contains("raw"));
    assert!(!debug.contains(&KERNEL_PHYS.get().to_string()));
    Ok(())
}

#[test]
fn public_mapping_debug_outputs_redact_addresses() -> Result<(), PageTableError> {
    let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
    let mapping = PageMapping::new(KERNEL_PHYS, flags);
    let visited = PageTableMapping::new(KERNEL_VIRT, mapping);
    let range = PageRangeMapping::new(KERNEL_PHYS, 2, flags);
    let translated = TranslatedRange::new(KERNEL_PHYS, 64, 1, flags);
    let entry = X86_64PageTableEntry::from_mapping(mapping)?;

    assert_debug_redacts_addresses(&format!("{mapping:?}"));
    assert_debug_redacts_addresses(&format!("{visited:?}"));
    assert_debug_redacts_addresses(&format!("{range:?}"));
    assert_debug_redacts_addresses(&format!("{translated:?}"));
    assert_debug_redacts_addresses(&format!("{:?}", TlbFlush::Page(KERNEL_VIRT)));
    assert_debug_redacts_addresses(&format!("{entry:?}"));
    Ok(())
}

#[test]
fn outcome_debug_outputs_redact_mapping_addresses() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let initial = GenericPageFlags::kernel(PageAccess::ReadWrite);
    let protected = GenericPageFlags::kernel(PageAccess::ReadOnly);

    let map = mapper.map_page(KERNEL_VIRT, KERNEL_PHYS, initial)?;
    let protect = mapper.protect_page(KERNEL_VIRT, protected)?;
    let unmap = mapper.unmap_page(KERNEL_VIRT)?;

    assert_debug_redacts_addresses(&format!("{map:?}"));
    assert_debug_redacts_addresses(&format!("{protect:?}"));
    assert_debug_redacts_addresses(&format!("{unmap:?}"));
    Ok(())
}

fn assert_debug_redacts_addresses(debug: &str) {
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("PhysAddr"));
    assert!(!debug.contains("VirtAddr"));
    assert!(!debug.contains(&KERNEL_PHYS.get().to_string()));
    assert!(!debug.contains(&KERNEL_VIRT.get().to_string()));
}
