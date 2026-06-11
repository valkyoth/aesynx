use alloc::{format, string::ToString};

use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};
use crate::page_table::{PageTableError, PageTableMapper};

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
