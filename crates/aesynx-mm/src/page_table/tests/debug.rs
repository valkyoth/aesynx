use alloc::{format, string::ToString};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{
    PageMapping, PageRangeMapping, PageTableError, PageTableMapper, PageTableMapping,
    PageTableRoot, PageTableSlot, TlbFlush, TranslatedRange, X86_64PageTableEntry,
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
    assert_debug_hides_addresses(&debug);
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
    assert_debug_hides_addresses(&debug);
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
    let leaf_slot = PageTableSlot::leaf(mapping)?;

    assert_debug_redacts_addresses(&format!("{mapping:?}"));
    assert_debug_redacts_addresses(&format!("{visited:?}"));
    assert_debug_redacts_addresses(&format!("{range:?}"));
    assert_debug_redacts_addresses(&format!("{translated:?}"));
    assert_debug_redacts_addresses(&format!("{:?}", TlbFlush::Page(KERNEL_VIRT)));
    assert_debug_redacts_addresses(&format!("{entry:?}"));
    let leaf_slot_debug = format!("{leaf_slot:?}");
    assert!(leaf_slot_debug.contains("leaf-or-corrupt"));
    assert!(!leaf_slot_debug.contains("raw"));
    assert_debug_hides_addresses(&leaf_slot_debug);
    Ok(())
}

#[test]
fn root_debug_redacts_model_internals_without_physical_claims() {
    let root = PageTableRoot::new(0);
    let debug = format!("{root:?}");

    assert!(debug.contains("PageTableRoot"));
    assert!(debug.contains("model-root"));
    assert!(!debug.contains("model_table_index"));
    assert!(!debug.contains("table_index"));
    assert!(!debug.contains("PhysAddr"));
    assert!(!debug.contains("PhysFrame"));
    assert!(!debug.contains("cr3"));
    assert!(!debug.contains("physical"));
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

#[test]
fn range_outcome_debug_outputs_are_aggregate_only() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    let initial = GenericPageFlags::kernel(PageAccess::ReadWrite);
    let protected = GenericPageFlags::kernel(PageAccess::ReadOnly);

    let map = mapper.map_contiguous(KERNEL_VIRT, KERNEL_PHYS, 2, initial)?;
    let protect = mapper.protect_contiguous(KERNEL_VIRT, 2, protected)?;
    let unmap = mapper.unmap_contiguous(KERNEL_VIRT, 2)?;

    for debug in [
        format!("{map:?}"),
        format!("{protect:?}"),
        format!("{unmap:?}"),
    ] {
        assert!(debug.contains("pages"));
        assert!(debug.contains("flush"));
        assert!(!debug.contains("<redacted>"));
        assert!(!debug.contains("PageMapping"));
        assert!(!debug.contains("PhysAddr"));
        assert!(!debug.contains("VirtAddr"));
        assert_debug_hides_addresses(&debug);
    }
    Ok(())
}

#[test]
fn mapper_report_debug_outputs_are_aggregate_only() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<4>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;

    let status_debug = format!("{:?}", mapper.status_checked()?);
    let audit_debug = format!("{:?}", mapper.audit()?);
    let summary_debug = format!("{:?}", mapper.mapping_summary()?);

    for debug in [status_debug, audit_debug, summary_debug] {
        assert!(debug.contains("tables") || debug.contains("pages"));
        assert!(!debug.contains("<redacted>"));
        assert!(!debug.contains("slots"));
        assert!(!debug.contains("raw"));
        assert_debug_hides_addresses(&debug);
    }
    Ok(())
}

fn assert_debug_redacts_addresses(debug: &str) {
    assert!(debug.contains("<redacted>"));
    assert_debug_hides_addresses(debug);
}

fn assert_debug_hides_addresses(debug: &str) {
    assert!(!debug.contains("PhysAddr"));
    assert!(!debug.contains("VirtAddr"));
    assert!(!debug.contains(&KERNEL_PHYS.get().to_string()));
    assert!(!debug.contains(&KERNEL_VIRT.get().to_string()));
}
