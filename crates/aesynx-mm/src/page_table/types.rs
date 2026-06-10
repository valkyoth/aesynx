use aesynx_abi::{PhysAddr, VirtAddr};

use crate::GenericPageFlags;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageTableError {
    EmptyArena,
    InvalidPageCount,
    InvalidByteCount,
    InvalidVirtualAddress,
    InvalidPhysicalAddress,
    UnalignedVirtualAddress,
    UnalignedPhysicalAddress,
    InvalidMappingFlags,
    AlreadyMapped,
    NotMapped,
    NonContiguousRange,
    UnexpectedMappingFlags,
    UnexpectedVirtualAddressSpace,
    PhysicalAlias,
    RangeTooLarge,
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

impl TlbFlush {
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::AddressSpace, _) | (_, Self::AddressSpace) => Self::AddressSpace,
            (Self::None, flush) | (flush, Self::None) => flush,
            (Self::Page(left), Self::Page(right)) if left == right => Self::Page(left),
            (Self::Page(_), Self::Page(_)) => Self::AddressSpace,
        }
    }
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
pub struct PageTableMapping {
    virt: VirtAddr,
    mapping: PageMapping,
}

impl PageTableMapping {
    #[must_use]
    pub const fn new(virt: VirtAddr, mapping: PageMapping) -> Self {
        Self { virt, mapping }
    }

    #[must_use]
    pub const fn virt(self) -> VirtAddr {
        self.virt
    }

    #[must_use]
    pub const fn mapping(self) -> PageMapping {
        self.mapping
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRangeMapping {
    start_phys: PhysAddr,
    pages: u64,
    flags: GenericPageFlags,
}

impl PageRangeMapping {
    #[must_use]
    pub const fn new(start_phys: PhysAddr, pages: u64, flags: GenericPageFlags) -> Self {
        Self {
            start_phys,
            pages,
            flags,
        }
    }

    #[must_use]
    pub const fn start_phys(self) -> PhysAddr {
        self.start_phys
    }

    #[must_use]
    pub const fn pages(self) -> u64 {
        self.pages
    }

    #[must_use]
    pub const fn flags(self) -> GenericPageFlags {
        self.flags
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TranslatedRange {
    start_phys: PhysAddr,
    byte_len: u64,
    pages: u64,
    flags: GenericPageFlags,
}

impl TranslatedRange {
    #[must_use]
    pub const fn new(
        start_phys: PhysAddr,
        byte_len: u64,
        pages: u64,
        flags: GenericPageFlags,
    ) -> Self {
        Self {
            start_phys,
            byte_len,
            pages,
            flags,
        }
    }

    #[must_use]
    pub const fn start_phys(self) -> PhysAddr {
        self.start_phys
    }

    #[must_use]
    pub const fn byte_len(self) -> u64 {
        self.byte_len
    }

    #[must_use]
    pub const fn pages(self) -> u64 {
        self.pages
    }

    #[must_use]
    pub const fn flags(self) -> GenericPageFlags {
        self.flags
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageTableRoot {
    table_index: usize,
}

impl PageTableRoot {
    #[must_use]
    pub(crate) const fn new(table_index: usize) -> Self {
        Self { table_index }
    }

    #[must_use]
    pub const fn table_index(self) -> usize {
        self.table_index
    }
}
