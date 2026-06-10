use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{FRAME_SIZE, GenericPageFlags};

pub const PAGE_TABLE_ENTRIES: usize = 512;
pub const PAGE_TABLE_LEVELS: usize = 4;

const PAGE_OFFSET_MASK: u64 = FRAME_SIZE - 1;
const CANONICAL_LOW_END: u64 = 0x0000_7fff_ffff_ffff;
const CANONICAL_HIGH_START: u64 = 0xffff_8000_0000_0000;
const MAX_PHYSICAL_ADDR: u64 = 0x000f_ffff_ffff_ffff;
const LEVEL_SHIFTS: [u64; PAGE_TABLE_LEVELS] = [39, 30, 21, 12];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageTableError {
    EmptyArena,
    InvalidVirtualAddress,
    InvalidPhysicalAddress,
    UnalignedVirtualAddress,
    UnalignedPhysicalAddress,
    InvalidMappingFlags,
    AlreadyMapped,
    NotMapped,
    OutOfPageTables,
    CorruptTable,
    AddressOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlbFlush {
    None,
    Page(VirtAddr),
    AddressSpace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageMapping {
    phys: PhysAddr,
    flags: GenericPageFlags,
}

impl PageMapping {
    #[must_use]
    pub const fn new(phys: PhysAddr, flags: GenericPageFlags) -> Self {
        Self { phys, flags }
    }

    #[must_use]
    pub const fn phys(self) -> PhysAddr {
        self.phys
    }

    #[must_use]
    pub const fn flags(self) -> GenericPageFlags {
        self.flags
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MapOutcome {
    flush: TlbFlush,
}

impl MapOutcome {
    #[must_use]
    pub const fn new(flush: TlbFlush) -> Self {
        Self { flush }
    }

    #[must_use]
    pub const fn flush(self) -> TlbFlush {
        self.flush
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnmapOutcome {
    mapping: PageMapping,
    flush: TlbFlush,
}

impl UnmapOutcome {
    #[must_use]
    pub const fn new(mapping: PageMapping, flush: TlbFlush) -> Self {
        Self { mapping, flush }
    }

    #[must_use]
    pub const fn mapping(self) -> PageMapping {
        self.mapping
    }

    #[must_use]
    pub const fn flush(self) -> TlbFlush {
        self.flush
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageTableStatus {
    pub total_tables: u64,
    pub used_tables: u64,
    pub mapped_pages: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct X86_64PageTableEntry {
    raw: u64,
}

impl X86_64PageTableEntry {
    const PRESENT: u64 = 1 << 0;
    const WRITABLE: u64 = 1 << 1;
    const USER: u64 = 1 << 2;
    const WRITE_THROUGH: u64 = 1 << 3;
    const CACHE_DISABLE: u64 = 1 << 4;
    const GLOBAL: u64 = 1 << 8;
    const NO_EXECUTE: u64 = 1 << 63;
    const ADDRESS_MASK: u64 = 0x000f_ffff_ffff_f000;
    const SOFTWARE_NEXT_TABLE: u64 = 1 << 9;

    #[must_use]
    pub const fn empty() -> Self {
        Self { raw: 0 }
    }

    pub fn from_mapping(mapping: PageMapping) -> Result<Self, PageTableError> {
        validate_phys(mapping.phys())?;
        if mapping.flags().device_memory && mapping.flags().access.executable() {
            return Err(PageTableError::InvalidMappingFlags);
        }
        let mut raw = (mapping.phys().get() & Self::ADDRESS_MASK) | Self::PRESENT;
        if mapping.flags().access.writable() {
            raw |= Self::WRITABLE;
        }
        if matches!(mapping.flags().privilege, crate::PagePrivilege::User) {
            raw |= Self::USER;
        }
        if mapping.flags().is_global() {
            raw |= Self::GLOBAL;
        }
        if !mapping.flags().access.executable() {
            raw |= Self::NO_EXECUTE;
        }
        if mapping.flags().device_memory {
            raw |= Self::WRITE_THROUGH | Self::CACHE_DISABLE;
        }
        Ok(Self { raw })
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PageTableSlot {
    raw: u64,
}

impl PageTableSlot {
    const EMPTY: Self = Self { raw: 0 };

    fn next(index: usize) -> Result<Self, PageTableError> {
        let encoded = (index as u64)
            .checked_shl(12)
            .ok_or(PageTableError::AddressOverflow)?;
        Ok(Self {
            raw: X86_64PageTableEntry::PRESENT
                | X86_64PageTableEntry::SOFTWARE_NEXT_TABLE
                | encoded,
        })
    }

    fn leaf(mapping: PageMapping) -> Result<Self, PageTableError> {
        Ok(Self {
            raw: X86_64PageTableEntry::from_mapping(mapping)?.raw(),
        })
    }

    const fn is_empty(self) -> bool {
        self.raw == 0
    }

    const fn is_next(self) -> bool {
        self.raw & X86_64PageTableEntry::SOFTWARE_NEXT_TABLE != 0
    }

    fn next_index(self) -> Result<usize, PageTableError> {
        if !self.is_next() {
            return Err(PageTableError::CorruptTable);
        }
        usize::try_from((self.raw & X86_64PageTableEntry::ADDRESS_MASK) >> 12)
            .map_err(|_error| PageTableError::CorruptTable)
    }

    fn mapping(self) -> Option<PageMapping> {
        if self.is_empty() || self.is_next() || self.raw & X86_64PageTableEntry::PRESENT == 0 {
            return None;
        }

        let phys = PhysAddr::new(self.raw & X86_64PageTableEntry::ADDRESS_MASK);
        let writable = self.raw & X86_64PageTableEntry::WRITABLE != 0;
        let executable = self.raw & X86_64PageTableEntry::NO_EXECUTE == 0;
        if writable && executable {
            return None;
        }
        let access = if writable {
            crate::PageAccess::ReadWrite
        } else if executable {
            crate::PageAccess::ReadExecute
        } else {
            crate::PageAccess::ReadOnly
        };
        let mut flags = if self.raw & X86_64PageTableEntry::USER != 0 {
            GenericPageFlags::user(access)
        } else {
            GenericPageFlags::kernel(access)
        };
        if self.raw & (X86_64PageTableEntry::WRITE_THROUGH | X86_64PageTableEntry::CACHE_DISABLE)
            != 0
        {
            flags = flags.device();
        }
        if self.raw & X86_64PageTableEntry::GLOBAL != 0 {
            flags = flags.with_global().ok()?;
        }

        Some(PageMapping::new(phys, flags))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PageTable {
    slots: [PageTableSlot; PAGE_TABLE_ENTRIES],
}

impl PageTable {
    const EMPTY: Self = Self {
        slots: [PageTableSlot::EMPTY; PAGE_TABLE_ENTRIES],
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageTableMapper<const TABLES: usize> {
    tables: [PageTable; TABLES],
    used: [bool; TABLES],
    mapped_pages: u64,
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

    pub fn map_page(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: GenericPageFlags,
    ) -> Result<MapOutcome, PageTableError> {
        validate_virt_page(virt)?;
        validate_phys(phys)?;
        self.validate_map_capacity(virt)?;
        let indices = page_indices(virt);
        let mut table_index = 0usize;
        for slot_index in indices.iter().take(PAGE_TABLE_LEVELS - 1) {
            table_index = self.ensure_next_table(table_index, *slot_index)?;
        }
        let slot = &mut self.tables[table_index].slots[indices[PAGE_TABLE_LEVELS - 1]];
        if !slot.is_empty() {
            return Err(PageTableError::AlreadyMapped);
        }
        *slot = PageTableSlot::leaf(PageMapping::new(phys, flags))?;
        self.mapped_pages = self
            .mapped_pages
            .checked_add(1)
            .ok_or(PageTableError::AddressOverflow)?;
        Ok(MapOutcome::new(TlbFlush::Page(virt)))
    }

    pub fn unmap_page(&mut self, virt: VirtAddr) -> Result<UnmapOutcome, PageTableError> {
        validate_virt_page(virt)?;
        let indices = page_indices(virt);
        let table_index = self.leaf_table_index(indices)?;
        let slot = &mut self.tables[table_index].slots[indices[PAGE_TABLE_LEVELS - 1]];
        let mapping = slot.mapping().ok_or(PageTableError::NotMapped)?;
        *slot = PageTableSlot::EMPTY;
        self.mapped_pages -= 1;
        Ok(UnmapOutcome::new(mapping, TlbFlush::Page(virt)))
    }

    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        if !is_canonical(virt.get()) {
            return None;
        }
        let indices = page_indices(virt);
        let table_index = self.leaf_table_index(indices).ok()?;
        let mapping = self.tables[table_index].slots[indices[PAGE_TABLE_LEVELS - 1]].mapping()?;
        let offset = virt.get() & PAGE_OFFSET_MASK;
        mapping.phys().get().checked_add(offset).map(PhysAddr::new)
    }

    #[must_use]
    pub fn status(&self) -> PageTableStatus {
        PageTableStatus {
            total_tables: TABLES as u64,
            used_tables: self.used_tables(),
            mapped_pages: self.mapped_pages,
        }
    }

    fn validate_map_capacity(&self, virt: VirtAddr) -> Result<(), PageTableError> {
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
        let slot = self.tables[table_index].slots[slot_index];
        if slot.is_next() {
            return slot.next_index();
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

    fn leaf_table_index(
        &self,
        indices: [usize; PAGE_TABLE_LEVELS],
    ) -> Result<usize, PageTableError> {
        let mut table_index = 0usize;
        for slot_index in indices.iter().take(PAGE_TABLE_LEVELS - 1) {
            let slot = self.tables[table_index].slots[*slot_index];
            if slot.is_next() {
                let next = slot.next_index()?;
                if next < TABLES && self.used[next] {
                    table_index = next;
                } else {
                    return Err(PageTableError::CorruptTable);
                }
            } else if slot.is_empty() {
                return Err(PageTableError::NotMapped);
            } else {
                return Err(PageTableError::CorruptTable);
            }
        }
        Ok(table_index)
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
}

fn validate_virt_page(virt: VirtAddr) -> Result<(), PageTableError> {
    if !is_canonical(virt.get()) {
        return Err(PageTableError::InvalidVirtualAddress);
    }
    if virt.get() & PAGE_OFFSET_MASK != 0 {
        return Err(PageTableError::UnalignedVirtualAddress);
    }
    Ok(())
}

fn validate_phys(phys: PhysAddr) -> Result<(), PageTableError> {
    if phys.get() > MAX_PHYSICAL_ADDR {
        return Err(PageTableError::InvalidPhysicalAddress);
    }
    if phys.get() & PAGE_OFFSET_MASK != 0 {
        return Err(PageTableError::UnalignedPhysicalAddress);
    }
    Ok(())
}

const fn is_canonical(value: u64) -> bool {
    value <= CANONICAL_LOW_END || value >= CANONICAL_HIGH_START
}

fn page_indices(virt: VirtAddr) -> [usize; PAGE_TABLE_LEVELS] {
    [
        ((virt.get() >> LEVEL_SHIFTS[0]) & 0x1ff) as usize,
        ((virt.get() >> LEVEL_SHIFTS[1]) & 0x1ff) as usize,
        ((virt.get() >> LEVEL_SHIFTS[2]) & 0x1ff) as usize,
        ((virt.get() >> LEVEL_SHIFTS[3]) & 0x1ff) as usize,
    ]
}

#[cfg(test)]
mod tests;
