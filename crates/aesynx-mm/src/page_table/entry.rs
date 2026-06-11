use core::fmt;

use aesynx_abi::PhysAddr;

use crate::{FRAME_SIZE, GenericPageFlags};

use super::address::validate_phys;
use super::{PageMapping, PageTableError};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct X86_64PageTableEntry {
    raw: u64,
}

impl fmt::Debug for X86_64PageTableEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("X86_64PageTableEntry")
            .field("raw", &"<redacted>")
            .finish()
    }
}

impl X86_64PageTableEntry {
    pub(super) const PRESENT: u64 = 1 << 0;
    const WRITABLE: u64 = 1 << 1;
    const USER: u64 = 1 << 2;
    const WRITE_THROUGH: u64 = 1 << 3;
    const CACHE_DISABLE: u64 = 1 << 4;
    const GLOBAL: u64 = 1 << 8;
    const NO_EXECUTE: u64 = 1 << 63;
    pub(super) const ADDRESS_MASK: u64 = 0x000f_ffff_ffff_f000;
    pub(super) const SOFTWARE_NEXT_TABLE: u64 = 1 << 9;
    const CACHE_POLICY_MASK: u64 = Self::WRITE_THROUGH | Self::CACHE_DISABLE;
    const ALLOWED_LEAF_BITS: u64 = Self::PRESENT
        | Self::WRITABLE
        | Self::USER
        | Self::WRITE_THROUGH
        | Self::CACHE_DISABLE
        | Self::GLOBAL
        | Self::NO_EXECUTE
        | Self::ADDRESS_MASK;

    pub fn from_mapping(mapping: PageMapping) -> Result<Self, PageTableError> {
        validate_phys(mapping.phys())?;
        if mapping.flags().is_device_memory() && mapping.flags().access.executable() {
            return Err(PageTableError::InvalidMappingFlags);
        }
        if mapping.flags().is_global()
            && matches!(mapping.flags().privilege, crate::PagePrivilege::User)
        {
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
        if mapping.flags().is_device_memory() {
            raw |= Self::WRITE_THROUGH | Self::CACHE_DISABLE;
        }
        Ok(Self { raw })
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.raw
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct PageTableSlot {
    pub(crate) raw: u64,
}

impl fmt::Debug for PageTableSlot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PageTableSlot")
            .field("state", &self.debug_state())
            .finish()
    }
}

impl PageTableSlot {
    pub(crate) const EMPTY: Self = Self { raw: 0 };

    pub(crate) fn next(index: usize) -> Result<Self, PageTableError> {
        let encoded = (index as u64)
            .checked_mul(FRAME_SIZE)
            .ok_or(PageTableError::AddressOverflow)?;
        if encoded & !X86_64PageTableEntry::ADDRESS_MASK != 0 {
            return Err(PageTableError::AddressOverflow);
        }
        Ok(Self {
            raw: X86_64PageTableEntry::PRESENT
                | X86_64PageTableEntry::SOFTWARE_NEXT_TABLE
                | encoded,
        })
    }

    pub(crate) fn leaf(mapping: PageMapping) -> Result<Self, PageTableError> {
        Ok(Self {
            raw: X86_64PageTableEntry::from_mapping(mapping)?.raw(),
        })
    }

    pub(crate) const fn is_empty(self) -> bool {
        self.raw == 0
    }

    pub(crate) const fn is_next(self) -> bool {
        self.raw & X86_64PageTableEntry::SOFTWARE_NEXT_TABLE != 0
    }

    pub(crate) fn next_index(self) -> Result<usize, PageTableError> {
        if !self.is_next() {
            return Err(PageTableError::CorruptTable);
        }
        usize::try_from((self.raw & X86_64PageTableEntry::ADDRESS_MASK) >> 12)
            .map_err(|_error| PageTableError::CorruptTable)
    }

    pub(crate) fn mapping(self) -> Option<PageMapping> {
        if self.is_empty() || self.is_next() || self.raw & X86_64PageTableEntry::PRESENT == 0 {
            return None;
        }
        if self.raw & !X86_64PageTableEntry::ALLOWED_LEAF_BITS != 0 {
            return None;
        }

        let phys = PhysAddr::new(self.raw & X86_64PageTableEntry::ADDRESS_MASK);
        let writable = self.raw & X86_64PageTableEntry::WRITABLE != 0;
        let executable = self.raw & X86_64PageTableEntry::NO_EXECUTE == 0;
        let cache_policy = self.raw & X86_64PageTableEntry::CACHE_POLICY_MASK;
        let device_memory = cache_policy == X86_64PageTableEntry::CACHE_POLICY_MASK;
        if cache_policy != 0 && !device_memory {
            return None;
        }
        if writable && executable {
            return None;
        }
        if device_memory && executable {
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
        if device_memory {
            flags = flags.device();
        }
        if self.raw & X86_64PageTableEntry::GLOBAL != 0 {
            flags = flags.with_global().ok()?;
        }

        Some(PageMapping::new(phys, flags))
    }

    pub(crate) fn leaf_mapping(self) -> Result<PageMapping, PageTableError> {
        if self.is_empty() {
            return Err(PageTableError::NotMapped);
        }
        if self.is_next() {
            return Err(PageTableError::CorruptTable);
        }
        self.mapping().ok_or(PageTableError::CorruptTable)
    }

    fn debug_state(self) -> &'static str {
        if self.is_empty() {
            "empty"
        } else if self.is_next() {
            "next"
        } else {
            "leaf-or-corrupt"
        }
    }
}
