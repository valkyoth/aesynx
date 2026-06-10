#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageTableSmokeStatus {
    pub total_tables: u64,
    pub used_tables: u64,
    pub mapped_pages_before_unmap: u64,
    pub mapped_pages_after_unmap: u64,
    pub translate_offset_ok: bool,
    pub mapping_lookup_ok: bool,
    pub presence_ok: bool,
    pub protect_ok: bool,
    pub protect_range_ok: bool,
    pub range_lookup_ok: bool,
    pub mapped_range_ok: bool,
    pub unmapped_range_ok: bool,
    pub kernel_range_ok: bool,
    pub user_range_ok: bool,
    pub write_protected_range_ok: bool,
    pub non_executable_range_ok: bool,
    pub executable_range_ok: bool,
    pub normal_memory_range_ok: bool,
    pub local_range_ok: bool,
    pub kernel_space_range_ok: bool,
    pub user_space_range_ok: bool,
    pub no_executable_ok: bool,
    pub no_writable_ok: bool,
    pub no_device_ok: bool,
    pub no_global_ok: bool,
    pub kernel_only_ok: bool,
    pub audit_ok: bool,
    pub visit_ok: bool,
    pub flags_ok: bool,
    pub reclaim_ok: bool,
    pub range_ok: bool,
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
const SMOKE_RANGE_VIRT: aesynx_abi::VirtAddr = aesynx_abi::VirtAddr::new(0xffff_9000_0000_4000);
const SMOKE_RANGE_PHYS: aesynx_abi::PhysAddr = aesynx_abi::PhysAddr::new(0x0020_4000);
const SMOKE_USER_RANGE_VIRT: aesynx_abi::VirtAddr =
    aesynx_abi::VirtAddr::new(0x0000_0000_0040_0000);
const SMOKE_OFFSET: u64 = 0x123;
const SMOKE_PAGE_TABLES: usize = aesynx_mm::PAGE_TABLE_LEVELS;

pub fn run() -> Result<PageTableSmokeStatus, PageTableSmokeError> {
    let mut mapper = aesynx_mm::PageTableMapper::<SMOKE_PAGE_TABLES>::new()
        .map_err(PageTableSmokeError::Mapper)?;
    let flags = aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadWrite);
    let protected_flags = aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadOnly);
    if mapper
        .is_page_mapped(SMOKE_VIRT)
        .map_err(PageTableSmokeError::Mapper)?
    {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
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
    if !mapper
        .is_page_mapped(SMOKE_VIRT)
        .map_err(PageTableSmokeError::Mapper)?
    {
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
    if mapper
        .is_page_mapped(SMOKE_VIRT)
        .map_err(PageTableSmokeError::Mapper)?
    {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    let after_unmap = mapper.status();
    if after_unmap.used_tables != 1 {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }

    let range_map = mapper
        .map_contiguous(SMOKE_RANGE_VIRT, SMOKE_RANGE_PHYS, 2, protected_flags)
        .map_err(PageTableSmokeError::Mapper)?;
    if range_map.pages() != 2 || range_map.flush() != aesynx_mm::TlbFlush::AddressSpace {
        return Err(PageTableSmokeError::FlushMismatch);
    }
    let range_mapping = mapper
        .mapping_for_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    if range_mapping.start_phys() != SMOKE_RANGE_PHYS
        || range_mapping.pages() != 2
        || range_mapping.flags() != protected_flags
    {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    mapper
        .ensure_mapped_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_non_executable_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_no_executable_mappings()
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_no_writable_mappings()
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_no_device_mappings()
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_no_global_mappings()
        .map_err(PageTableSmokeError::Mapper)?;
    let range_execute_flags =
        aesynx_mm::GenericPageFlags::kernel(aesynx_mm::PageAccess::ReadExecute);
    let range_protect = mapper
        .protect_contiguous(SMOKE_RANGE_VIRT, 2, range_execute_flags)
        .map_err(PageTableSmokeError::Mapper)?;
    if range_protect.pages() != 2 || range_protect.flush() != aesynx_mm::TlbFlush::AddressSpace {
        return Err(PageTableSmokeError::FlushMismatch);
    }
    if mapper.translate(aesynx_abi::VirtAddr::new(SMOKE_RANGE_VIRT.get() + 0x1000))
        != Some(aesynx_abi::PhysAddr::new(SMOKE_RANGE_PHYS.get() + 0x1000))
    {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    mapper
        .ensure_contiguous_flags(SMOKE_RANGE_VIRT, 2, range_execute_flags)
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_kernel_mapped_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_write_protected_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_executable_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_normal_memory_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_local_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_kernel_space_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_no_user_mappings()
        .map_err(PageTableSmokeError::Mapper)?;
    let mut visited_range_pages = 0u64;
    let visited_pages = mapper
        .visit_mappings(|entry| {
            let mapping = entry.mapping();
            if entry.virt() == SMOKE_RANGE_VIRT
                && mapping == aesynx_mm::PageMapping::new(SMOKE_RANGE_PHYS, range_execute_flags)
            {
                visited_range_pages += 1;
            } else if entry.virt() == aesynx_abi::VirtAddr::new(SMOKE_RANGE_VIRT.get() + 0x1000)
                && mapping
                    == aesynx_mm::PageMapping::new(
                        aesynx_abi::PhysAddr::new(SMOKE_RANGE_PHYS.get() + 0x1000),
                        range_execute_flags,
                    )
            {
                visited_range_pages += 1;
            }
            Ok(())
        })
        .map_err(PageTableSmokeError::Mapper)?;
    if visited_pages != 2 || visited_range_pages != 2 {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    let range_unmap = mapper
        .unmap_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    if range_unmap.pages() != 2 || range_unmap.flush() != aesynx_mm::TlbFlush::AddressSpace {
        return Err(PageTableSmokeError::FlushMismatch);
    }
    if mapper.translate(SMOKE_RANGE_VIRT).is_some() {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    if mapper.ensure_mapped_contiguous(SMOKE_RANGE_VIRT, 2).is_ok() {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    mapper
        .ensure_unmapped_contiguous(SMOKE_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    let after_range = mapper.status();
    if after_range.used_tables != 1 || after_range.mapped_pages != 0 {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }
    let audit = mapper.audit().map_err(PageTableSmokeError::Mapper)?;
    if audit.total_tables() != after_range.total_tables
        || audit.used_tables() != after_range.used_tables
        || audit.reachable_tables() != after_range.used_tables
        || audit.mapped_pages() != after_range.mapped_pages
    {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }

    let user_flags = aesynx_mm::GenericPageFlags::user(aesynx_mm::PageAccess::ReadOnly);
    let user_range_map = mapper
        .map_contiguous(SMOKE_USER_RANGE_VIRT, SMOKE_RANGE_PHYS, 2, user_flags)
        .map_err(PageTableSmokeError::Mapper)?;
    if user_range_map.pages() != 2 || user_range_map.flush() != aesynx_mm::TlbFlush::AddressSpace {
        return Err(PageTableSmokeError::FlushMismatch);
    }
    mapper
        .ensure_user_mapped_contiguous(SMOKE_USER_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    mapper
        .ensure_user_space_contiguous(SMOKE_USER_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    let user_range_unmap = mapper
        .unmap_contiguous(SMOKE_USER_RANGE_VIRT, 2)
        .map_err(PageTableSmokeError::Mapper)?;
    if user_range_unmap.pages() != 2
        || user_range_unmap.flush() != aesynx_mm::TlbFlush::AddressSpace
    {
        return Err(PageTableSmokeError::FlushMismatch);
    }
    let final_status = mapper.status();
    if final_status.used_tables != 1 || final_status.mapped_pages != 0 {
        return Err(PageTableSmokeError::UnexpectedTranslation);
    }

    Ok(PageTableSmokeStatus {
        total_tables: after_unmap.total_tables,
        used_tables: after_range.used_tables,
        mapped_pages_before_unmap: before_unmap.mapped_pages,
        mapped_pages_after_unmap: after_range.mapped_pages,
        translate_offset_ok: true,
        mapping_lookup_ok: true,
        presence_ok: true,
        protect_ok: true,
        protect_range_ok: true,
        range_lookup_ok: true,
        mapped_range_ok: true,
        unmapped_range_ok: true,
        kernel_range_ok: true,
        user_range_ok: true,
        write_protected_range_ok: true,
        non_executable_range_ok: true,
        executable_range_ok: true,
        normal_memory_range_ok: true,
        local_range_ok: true,
        kernel_space_range_ok: true,
        user_space_range_ok: true,
        no_executable_ok: true,
        no_writable_ok: true,
        no_device_ok: true,
        no_global_ok: true,
        kernel_only_ok: true,
        audit_ok: true,
        visit_ok: true,
        flags_ok: true,
        reclaim_ok: true,
        range_ok: true,
        flush_page: true,
    })
}
