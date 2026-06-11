use core::fmt;

use aesynx_abi::{PhysAddr, VirtAddr};

use crate::GenericPageFlags;

mod address;
mod audit;
mod entry;
mod outcome;
mod policy;
mod preflight;
mod presence;
mod range;
mod range_policy;
mod range_translation;
mod report;
mod root;
mod status;
mod summary;
mod types;
mod walk;

use address::{PAGE_OFFSET_MASK, is_canonical, page_indices, validate_phys, validate_virt_page};
use entry::{PageTableSlot, X86_64PageTableEntry};
pub use outcome::{
    MapOutcome, MapRangeOutcome, ProtectOutcome, ProtectRangeOutcome, UnmapOutcome,
    UnmapRangeOutcome,
};
pub use report::{PageTableAudit, PageTableMappingSummary, PageTableStatus};
pub use types::{
    PageMapping, PageRangeMapping, PageTableError, PageTableMapping, PageTableRoot, TlbFlush,
    TranslatedRange,
};

pub const PAGE_TABLE_ENTRIES: usize = 512;
pub const PAGE_TABLE_LEVELS: usize = 4;

const MAX_NEXT_TABLE_INDEX: u64 = X86_64PageTableEntry::ADDRESS_MASK >> 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PageTable {
    slots: [PageTableSlot; PAGE_TABLE_ENTRIES],
}

impl PageTable {
    const EMPTY: Self = Self {
        slots: [PageTableSlot::EMPTY; PAGE_TABLE_ENTRIES],
    };

    fn is_empty(self) -> bool {
        let mut index = 0usize;
        while index < PAGE_TABLE_ENTRIES {
            if !self.slots[index].is_empty() {
                return false;
            }
            index += 1;
        }
        true
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PageTableMapper<const TABLES: usize> {
    tables: [PageTable; TABLES],
    used: [bool; TABLES],
    mapped_pages: u64,
}

impl<const TABLES: usize> fmt::Debug for PageTableMapper<TABLES> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PageTableMapper")
            .field("total_tables", &(TABLES as u64))
            .field("used_tables", &self.used_tables())
            .field("mapped_pages", &self.mapped_pages)
            .field("audit_ok", &self.audit().is_ok())
            .finish()
    }
}

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn new() -> Result<Self, PageTableError> {
        if TABLES == 0 {
            return Err(PageTableError::EmptyArena);
        }
        let mut used = [false; TABLES];
        used[0] = true;
        Ok(Self {
            tables: [PageTable::EMPTY; TABLES],
            used,
            mapped_pages: 0,
        })
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn root_table(&self) -> PageTableRoot {
        PageTableRoot::new(0)
    }

    pub fn map_page(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: GenericPageFlags,
    ) -> Result<MapOutcome, PageTableError> {
        validate_virt_page(virt)?;
        let mapping = PageMapping::new_checked(phys, flags)?;
        let leaf = PageTableSlot::leaf(mapping)?;
        self.audit()?;
        self.validate_map_capacity(virt)?;
        let mapped_pages = self
            .mapped_pages
            .checked_add(1)
            .ok_or(PageTableError::AddressOverflow)?;
        let indices = page_indices(virt);
        let mut table_index = 0usize;
        for slot_index in indices.iter().take(PAGE_TABLE_LEVELS - 1) {
            table_index = self.ensure_next_table(table_index, *slot_index)?;
        }
        let slot = &mut self.tables[table_index].slots[indices[PAGE_TABLE_LEVELS - 1]];
        if !slot.is_empty() {
            return Err(PageTableError::AlreadyMapped);
        }
        *slot = leaf;
        self.mapped_pages = mapped_pages;
        Ok(MapOutcome::new(TlbFlush::Page(virt)))
    }

    pub fn unmap_page(&mut self, virt: VirtAddr) -> Result<UnmapOutcome, PageTableError> {
        validate_virt_page(virt)?;
        self.audit()?;
        let indices = page_indices(virt);
        let path = self.table_path(indices)?;
        let table_index = path[PAGE_TABLE_LEVELS - 1];
        let slot = &mut self.tables[table_index].slots[indices[PAGE_TABLE_LEVELS - 1]];
        let mapping = slot.leaf_mapping()?;
        let mapped_pages = self
            .mapped_pages
            .checked_sub(1)
            .ok_or(PageTableError::CorruptTable)?;
        *slot = PageTableSlot::EMPTY;
        self.mapped_pages = mapped_pages;
        self.reclaim_empty_tables(indices, path)?;
        Ok(UnmapOutcome::new(
            mapping,
            flush_for_removed_mapping(virt, mapping),
        ))
    }

    pub fn protect_page(
        &mut self,
        virt: VirtAddr,
        flags: GenericPageFlags,
    ) -> Result<ProtectOutcome, PageTableError> {
        validate_virt_page(virt)?;
        validate_mapping_flags(flags)?;
        self.audit()?;
        let indices = page_indices(virt);
        let table_index = self.table_path(indices)?[PAGE_TABLE_LEVELS - 1];
        let slot = &mut self.tables[table_index].slots[indices[PAGE_TABLE_LEVELS - 1]];
        let previous = slot.leaf_mapping()?;
        let current = PageMapping::new_checked(previous.phys(), flags)?;
        let replacement = PageTableSlot::leaf(current)?;
        *slot = replacement;
        Ok(ProtectOutcome::new(
            previous,
            current,
            flush_for_protect(virt, previous, current),
        ))
    }

    pub fn translate(&self, virt: VirtAddr) -> Result<PhysAddr, PageTableError> {
        self.translate_checked(virt)
    }

    pub fn translate_checked(&self, virt: VirtAddr) -> Result<PhysAddr, PageTableError> {
        if !is_canonical(virt.get()) {
            return Err(PageTableError::InvalidVirtualAddress);
        }
        self.audit()?;
        let page = VirtAddr::new(virt.get() & !PAGE_OFFSET_MASK);
        let mapping = self.mapping_for_address(page)?;
        let offset = virt.get() & PAGE_OFFSET_MASK;
        mapping
            .phys()
            .get()
            .checked_add(offset)
            .map(PhysAddr::new)
            .ok_or(PageTableError::AddressOverflow)
    }

    pub fn mapping_for_page(&self, virt: VirtAddr) -> Result<PageMapping, PageTableError> {
        validate_virt_page(virt)?;
        self.audit()?;
        self.mapping_for_address(virt)
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn status(&self) -> PageTableStatus {
        PageTableStatus::new(TABLES as u64, self.used_tables(), self.mapped_pages)
    }

    fn validate_map_capacity(&self, virt: VirtAddr) -> Result<(), PageTableError> {
        if TABLES == 0 {
            return Err(PageTableError::EmptyArena);
        }
        if (TABLES - 1) as u64 > MAX_NEXT_TABLE_INDEX {
            return Err(PageTableError::AddressOverflow);
        }
        let indices = page_indices(virt);
        let mut table_index = 0usize;
        let mut missing_tables = 0u64;
        for (level, slot_index) in indices.iter().enumerate().take(PAGE_TABLE_LEVELS - 1) {
            let slot = self.tables[table_index].slots[*slot_index];
            if slot.is_empty() {
                missing_tables += (PAGE_TABLE_LEVELS - 1 - level) as u64;
                break;
            }
            if slot.is_next() {
                table_index = slot.next_index()?;
                if table_index >= TABLES || !self.used[table_index] {
                    return Err(PageTableError::CorruptTable);
                }
            } else {
                return Err(PageTableError::AlreadyMapped);
            }
        }
        if missing_tables > self.free_tables() {
            return Err(PageTableError::OutOfPageTables);
        }
        if missing_tables == 0
            && !self.tables[table_index].slots[indices[PAGE_TABLE_LEVELS - 1]].is_empty()
        {
            return Err(PageTableError::AlreadyMapped);
        }
        Ok(())
    }

    fn ensure_next_table(
        &mut self,
        table_index: usize,
        slot_index: usize,
    ) -> Result<usize, PageTableError> {
        if table_index >= TABLES || slot_index >= PAGE_TABLE_ENTRIES {
            return Err(PageTableError::CorruptTable);
        }
        let slot = self.tables[table_index].slots[slot_index];
        if slot.is_next() {
            let next = slot.next_index()?;
            if next >= TABLES || !self.used[next] {
                return Err(PageTableError::CorruptTable);
            }
            return Ok(next);
        }
        if slot.is_empty() {
            let next = self.allocate_table()?;
            self.tables[table_index].slots[slot_index] = PageTableSlot::next(next)?;
            return Ok(next);
        }
        Err(PageTableError::AlreadyMapped)
    }

    fn allocate_table(&mut self) -> Result<usize, PageTableError> {
        let mut index = 1usize;
        while index < TABLES {
            if !self.used[index] {
                self.used[index] = true;
                self.tables[index] = PageTable::EMPTY;
                return Ok(index);
            }
            index += 1;
        }
        Err(PageTableError::OutOfPageTables)
    }

    fn used_tables(&self) -> u64 {
        let mut count = 0u64;
        let mut index = 0usize;
        while index < TABLES {
            if self.used[index] {
                count += 1;
            }
            index += 1;
        }
        count
    }

    fn free_tables(&self) -> u64 {
        TABLES as u64 - self.used_tables()
    }

    fn mapping_for_address(&self, virt: VirtAddr) -> Result<PageMapping, PageTableError> {
        let indices = page_indices(virt);
        let table_index = self.table_path(indices)?[PAGE_TABLE_LEVELS - 1];
        self.tables[table_index].slots[indices[PAGE_TABLE_LEVELS - 1]].leaf_mapping()
    }

    fn table_path(
        &self,
        indices: [usize; PAGE_TABLE_LEVELS],
    ) -> Result<[usize; PAGE_TABLE_LEVELS], PageTableError> {
        if TABLES == 0 {
            return Err(PageTableError::EmptyArena);
        }
        let mut path = [0usize; PAGE_TABLE_LEVELS];
        let mut table_index = 0usize;
        path[0] = table_index;
        for (level, slot_index) in indices.iter().enumerate().take(PAGE_TABLE_LEVELS - 1) {
            if *slot_index >= PAGE_TABLE_ENTRIES {
                return Err(PageTableError::CorruptTable);
            }
            let slot = self.tables[table_index].slots[*slot_index];
            if slot.is_next() {
                let next = slot.next_index()?;
                if next < TABLES && self.used[next] {
                    table_index = next;
                    path[level + 1] = table_index;
                } else {
                    return Err(PageTableError::CorruptTable);
                }
            } else if slot.is_empty() {
                return Err(PageTableError::NotMapped);
            } else {
                return Err(PageTableError::CorruptTable);
            }
        }
        Ok(path)
    }

    fn reclaim_empty_tables(
        &mut self,
        indices: [usize; PAGE_TABLE_LEVELS],
        path: [usize; PAGE_TABLE_LEVELS],
    ) -> Result<(), PageTableError> {
        let mut lowest_reclaim_level = PAGE_TABLE_LEVELS;
        let mut ignored_child_slot = None;
        let mut level = PAGE_TABLE_LEVELS - 1;
        while level > 0 {
            let table_index = path[level];
            let parent = path[level - 1];
            let parent_slot = indices[level - 1];
            self.validate_reclaim_step(table_index, parent, parent_slot)?;
            if !self.table_is_empty_except(table_index, ignored_child_slot) {
                break;
            }
            lowest_reclaim_level = level;
            ignored_child_slot = Some(parent_slot);
            level -= 1;
        }

        if lowest_reclaim_level == PAGE_TABLE_LEVELS {
            return Ok(());
        }

        let mut level = PAGE_TABLE_LEVELS - 1;
        while level >= lowest_reclaim_level {
            let table_index = path[level];
            let parent = path[level - 1];
            let parent_slot = indices[level - 1];
            self.tables[parent].slots[parent_slot] = PageTableSlot::EMPTY;
            self.tables[table_index] = PageTable::EMPTY;
            self.used[table_index] = false;
            if level == lowest_reclaim_level {
                break;
            }
            level -= 1;
        }
        Ok(())
    }

    fn table_is_empty_except(&self, table_index: usize, ignored_slot: Option<usize>) -> bool {
        let mut slot = 0usize;
        while slot < PAGE_TABLE_ENTRIES {
            if Some(slot) != ignored_slot && !self.tables[table_index].slots[slot].is_empty() {
                return false;
            }
            slot += 1;
        }
        true
    }

    fn validate_reclaim_step(
        &self,
        table_index: usize,
        parent: usize,
        parent_slot: usize,
    ) -> Result<(), PageTableError> {
        if table_index == 0
            || table_index >= TABLES
            || parent >= TABLES
            || parent_slot >= PAGE_TABLE_ENTRIES
            || !self.used[table_index]
            || !self.used[parent]
        {
            return Err(PageTableError::CorruptTable);
        }
        let slot = self.tables[parent].slots[parent_slot];
        if slot.next_index()? != table_index {
            return Err(PageTableError::CorruptTable);
        }
        Ok(())
    }
}

impl PageMapping {
    pub fn new_checked(phys: PhysAddr, flags: GenericPageFlags) -> Result<Self, PageTableError> {
        validate_phys(phys)?;
        validate_mapping_flags(flags)?;
        Ok(Self::new(phys, flags))
    }
}

fn validate_mapping_flags(flags: GenericPageFlags) -> Result<(), PageTableError> {
    PageTableSlot::leaf(PageMapping::new(PhysAddr::new(0), flags))?;
    Ok(())
}

fn flush_for_removed_mapping(virt: VirtAddr, mapping: PageMapping) -> TlbFlush {
    if mapping.flags().is_global() {
        TlbFlush::AddressSpace
    } else {
        TlbFlush::Page(virt)
    }
}

fn flush_for_protect(virt: VirtAddr, previous: PageMapping, current: PageMapping) -> TlbFlush {
    if previous.flags().is_global() || current.flags().is_global() {
        TlbFlush::AddressSpace
    } else {
        TlbFlush::Page(virt)
    }
}

#[cfg(test)]
mod tests;
