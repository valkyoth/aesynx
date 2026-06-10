use aesynx_abi::{PhysAddr, VirtAddr};

use crate::GenericPageFlags;

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
pub struct ProtectOutcome {
    previous: PageMapping,
    current: PageMapping,
    flush: TlbFlush,
}

impl ProtectOutcome {
    #[must_use]
    pub const fn new(previous: PageMapping, current: PageMapping, flush: TlbFlush) -> Self {
        Self {
            previous,
            current,
            flush,
        }
    }

    #[must_use]
    pub const fn previous(self) -> PageMapping {
        self.previous
    }

    #[must_use]
    pub const fn current(self) -> PageMapping {
        self.current
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
