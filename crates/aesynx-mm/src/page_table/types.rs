use core::fmt;

use aesynx_abi::{PhysAddr, VirtAddr};

use crate::GenericPageFlags;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageTableError {
    EmptyArena,
    EmptyAddressSpace,
    IncompleteAddressSpace,
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TlbFlush {
    None,
    Page(VirtAddr),
    AddressSpace,
}

impl fmt::Debug for TlbFlush {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("None"),
            Self::Page(_virt) => formatter.debug_tuple("Page").field(&"<redacted>").finish(),
            Self::AddressSpace => formatter.write_str("AddressSpace"),
        }
    }
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PageMapping {
    phys: PhysAddr,
    flags: GenericPageFlags,
}

impl fmt::Debug for PageMapping {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PageMapping")
            .field("phys", &"<redacted>")
            .field("flags", &self.flags)
            .finish()
    }
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PageTableMapping {
    virt: VirtAddr,
    mapping: PageMapping,
}

impl fmt::Debug for PageTableMapping {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PageTableMapping")
            .field("virt", &"<redacted>")
            .field("mapping", &self.mapping)
            .finish()
    }
}

impl PageTableMapping {
    #[must_use]
    pub(crate) const fn new(virt: VirtAddr, mapping: PageMapping) -> Self {
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PageRangeMapping {
    start_phys: PhysAddr,
    pages: u64,
    flags: GenericPageFlags,
}

impl fmt::Debug for PageRangeMapping {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PageRangeMapping")
            .field("start_phys", &"<redacted>")
            .field("pages", &self.pages)
            .field("flags", &self.flags)
            .finish()
    }
}

impl PageRangeMapping {
    #[must_use]
    pub(crate) const fn new(start_phys: PhysAddr, pages: u64, flags: GenericPageFlags) -> Self {
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TranslatedRange {
    start_phys: PhysAddr,
    byte_len: u64,
    pages: u64,
    flags: GenericPageFlags,
}

impl fmt::Debug for TranslatedRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TranslatedRange")
            .field("start_phys", &"<redacted>")
            .field("byte_len", &self.byte_len)
            .field("pages", &self.pages)
            .field("flags", &self.flags)
            .finish()
    }
}

impl TranslatedRange {
    #[must_use]
    pub(crate) const fn new(
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
