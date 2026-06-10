#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageTableSmokeStatus {
    pub total_tables: u64,
    pub used_tables: u64,
    pub mapped_pages_before_unmap: u64,
    pub mapped_pages_after_unmap: u64,
    pub translate_offset_ok: bool,
    pub mapping_lookup_ok: bool,
    pub protect_ok: bool,
    pub flush_page: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageTableSmokeError {
    Mapper(aesynx_mm::PageTableError),
    UnexpectedTranslation,
    FlushMismatch,
}

const SMOKE_VIRT: aesynx_abi::VirtAddr = aesynx_abi::VirtAddr::new(0xffff_9000_0000_0000);
const SMOKE_PHYS: aesynx_abi::PhysAddr = aesynx_abi::PhysAddr::new(0x0020_0000);
const SMOKE_OFFSET: u64 = 0x123;
const SMOKE_PAGE_TABLES: usize = aesynx_mm::PAGE_TABLE_LEVELS;

pub fn run() -> Result<PageTableSmokeStatus, PageTableSmokeError> {
    let mut mapper = aesynx_mm::PageTableMapper::<SMOKE_PAGE_TABLES>::new()
        .map_err(PageTableSmokeError::Mapper)?;
    let flags = aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadWrite);
    let protected_flags = aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadOnly);
    let map = mapper
        .map_page(SMOKE_VIRT, SMOKE_PHYS, flags)
        .map_err(PageTableSmokeError::Mapper)?;
    if map.flush() != aesynx_mm::TlbFlush::Page(SMOKE_VIRT) {
        return Err(PageTableSmokeError::FlushMismatch);
    }

    let translated = mapper
        .translate(aesynx_abi::VirtAddr::new(SMOKE_VIRT.get() + SMOKE_OFFSET))
        .ok_or(PageTableSmokeError::UnexpectedTranslation)?;
    if translated != aesynx_abi::PhysAddr::new(SMOKE_PHYS.get() + SMOKE_OFFSET) {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    let mapping = mapper
        .mapping_for_page(SMOKE_VIRT)
        .map_err(PageTableSmokeError::Mapper)?;
    if mapping.phys() != SMOKE_PHYS || mapping.flags() != flags {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    let protect = mapper
        .protect_page(SMOKE_VIRT, protected_flags)
        .map_err(PageTableSmokeError::Mapper)?;
    if protect.previous().flags() != flags
        || protect.current().flags() != protected_flags
        || protect.flush() != aesynx_mm::TlbFlush::Page(SMOKE_VIRT)
    {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }

    let before_unmap = mapper.status();
    let unmap = mapper
        .unmap_page(SMOKE_VIRT)
        .map_err(PageTableSmokeError::Mapper)?;
    if unmap.flush() != aesynx_mm::TlbFlush::Page(SMOKE_VIRT) {
        return Err(PageTableSmokeError::FlushMismatch);
    }
    if unmap.mapping().phys() != SMOKE_PHYS || unmap.mapping().flags() != protected_flags {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    if mapper.translate(SMOKE_VIRT).is_some() {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    let after_unmap = mapper.status();

    Ok(PageTableSmokeStatus {
        total_tables: after_unmap.total_tables,
        used_tables: after_unmap.used_tables,
        mapped_pages_before_unmap: before_unmap.mapped_pages,
        mapped_pages_after_unmap: after_unmap.mapped_pages,
        translate_offset_ok: true,
        mapping_lookup_ok: true,
        protect_ok: true,
        flush_page: true,
    })
}
