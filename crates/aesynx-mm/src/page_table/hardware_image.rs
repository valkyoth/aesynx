use core::fmt;

use aesynx_abi::PhysAddr;

use crate::{FRAME_SIZE, PagePrivilege};

use super::address::validate_phys;
use super::entry::X86_64PageTableEntry;
use super::{PAGE_TABLE_ENTRIES, PAGE_TABLE_LEVELS, PageTableError, PageTableMapper};

const HARDWARE_PRESENT: u64 = 1 << 0;
const HARDWARE_WRITABLE: u64 = 1 << 1;
const HARDWARE_USER: u64 = 1 << 2;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct X86_64PageTableImage<const TABLES: usize> {
    root_phys: PhysAddr,
    entries: [[u64; PAGE_TABLE_ENTRIES]; TABLES],
    used: [bool; TABLES],
    mapped_pages: u64,
}

impl<const TABLES: usize> fmt::Debug for X86_64PageTableImage<TABLES> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("X86_64PageTableImage")
            .field("root_phys", &"<redacted>")
            .field("tables", &(TABLES as u64))
            .field("used_tables", &self.used_tables())
            .field("mapped_pages", &self.mapped_pages)
            .finish()
    }
}

impl<const TABLES: usize> X86_64PageTableImage<TABLES> {
    #[must_use]
    pub const fn root_phys(self) -> PhysAddr {
        self.root_phys
    }

    #[must_use]
    pub const fn mapped_pages(self) -> u64 {
        self.mapped_pages
    }

    #[must_use]
    pub fn used_tables(self) -> u64 {
        count_used_tables(&self.used)
    }

    pub fn table_phys(self, index: usize) -> Result<PhysAddr, PageTableError> {
        if index >= TABLES || !self.used[index] {
            return Err(PageTableError::CorruptTable);
        }
        table_phys(self.root_phys, index)
    }

    pub fn copy_table_entries(
        self,
        index: usize,
        output: &mut [u64; PAGE_TABLE_ENTRIES],
    ) -> Result<(), PageTableError> {
        if index >= TABLES || !self.used[index] {
            return Err(PageTableError::CorruptTable);
        }
        *output = self.entries[index];
        Ok(())
    }
}

impl<const TABLES: usize, const MAPPED_FRAMES: usize> PageTableMapper<TABLES, MAPPED_FRAMES> {
    pub fn export_x86_64_hardware_image(
        &self,
        root_phys: PhysAddr,
    ) -> Result<X86_64PageTableImage<TABLES>, PageTableError> {
        validate_phys(root_phys)?;
        self.audit()?;
        validate_table_arena_phys(root_phys, TABLES)?;

        let mut entries = [[0u64; PAGE_TABLE_ENTRIES]; TABLES];
        let mut table = 0usize;
        while table < TABLES {
            if self.used[table] {
                let mut slot = 0usize;
                while slot < PAGE_TABLE_ENTRIES {
                    let model_slot = self.tables[table].slots[slot];
                    entries[table][slot] = if model_slot.is_empty() {
                        0
                    } else if model_slot.is_next() {
                        let next = model_slot.next_index()?;
                        if next >= TABLES || !self.used[next] {
                            return Err(PageTableError::CorruptTable);
                        }
                        let permissions = self.subtree_permissions(next, 1)?;
                        hardware_next_entry(table_phys(root_phys, next)?, permissions)
                    } else {
                        model_slot.leaf_mapping()?;
                        model_slot.raw
                    };
                    slot += 1;
                }
            }
            table += 1;
        }

        Ok(X86_64PageTableImage {
            root_phys,
            entries,
            used: self.used,
            mapped_pages: self.mapped_pages,
        })
    }

    fn subtree_permissions(
        &self,
        table: usize,
        depth: usize,
    ) -> Result<NextTablePermissions, PageTableError> {
        if table >= TABLES || !self.used[table] || depth >= PAGE_TABLE_LEVELS {
            return Err(PageTableError::CorruptTable);
        }

        let mut permissions = NextTablePermissions::empty();
        let mut slot = 0usize;
        while slot < PAGE_TABLE_ENTRIES {
            let model_slot = self.tables[table].slots[slot];
            if model_slot.is_next() {
                let next = model_slot.next_index()?;
                permissions = permissions.union(self.subtree_permissions(next, depth + 1)?);
            } else if let Some(mapping) = model_slot.mapping()? {
                permissions.observe_leaf(mapping.flags());
            }
            slot += 1;
        }
        Ok(permissions)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NextTablePermissions {
    writable: bool,
    user: bool,
}

impl NextTablePermissions {
    const fn empty() -> Self {
        Self {
            writable: false,
            user: false,
        }
    }

    fn observe_leaf(&mut self, flags: crate::GenericPageFlags) {
        self.writable |= flags.access().writable();
        self.user |= matches!(flags.privilege(), PagePrivilege::User);
    }

    const fn union(self, other: Self) -> Self {
        Self {
            writable: self.writable || other.writable,
            user: self.user || other.user,
        }
    }
}

fn hardware_next_entry(phys: PhysAddr, permissions: NextTablePermissions) -> u64 {
    let mut raw = (phys.get() & X86_64PageTableEntry::ADDRESS_MASK) | HARDWARE_PRESENT;
    if permissions.writable {
        raw |= HARDWARE_WRITABLE;
    }
    if permissions.user {
        raw |= HARDWARE_USER;
    }
    raw
}

fn validate_table_arena_phys(root_phys: PhysAddr, tables: usize) -> Result<(), PageTableError> {
    if tables == 0 {
        return Err(PageTableError::EmptyArena);
    }
    table_phys(root_phys, tables - 1).map(|_last| ())
}

fn table_phys(root_phys: PhysAddr, index: usize) -> Result<PhysAddr, PageTableError> {
    let offset = (index as u64)
        .checked_mul(FRAME_SIZE)
        .ok_or(PageTableError::AddressOverflow)?;
    let phys = root_phys
        .get()
        .checked_add(offset)
        .map(PhysAddr::new)
        .ok_or(PageTableError::AddressOverflow)?;
    validate_phys(phys)?;
    Ok(phys)
}

fn count_used_tables(used: &[bool]) -> u64 {
    let mut count = 0u64;
    let mut index = 0usize;
    while index < used.len() {
        if used[index] {
            count += 1;
        }
        index += 1;
    }
    count
}
