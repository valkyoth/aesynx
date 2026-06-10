use aesynx_abi::{PhysAddr, VirtAddr};

use crate::GenericPageFlags;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageTableError {
    EmptyArena,
    InvalidPageCount,
    InvalidVirtualAddress,
    InvalidPhysicalAddress,
    UnalignedVirtualAddress,
    UnalignedPhysicalAddress,
    InvalidMappingFlags,
    AlreadyMapped,
    NotMapped,
    NonContiguousRange,
    UnexpectedMappingFlags,
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
pub struct MapRangeOutcome {
    pages: u64,
    flush: TlbFlush,
}

impl MapRangeOutcome {
    #[must_use]
    pub const fn new(pages: u64, flush: TlbFlush) -> Self {
        Self { pages, flush }
    }

    #[must_use]
    pub const fn pages(self) -> u64 {
        self.pages
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
pub struct UnmapRangeOutcome {
    pages: u64,
    flush: TlbFlush,
}

impl UnmapRangeOutcome {
    #[must_use]
    pub const fn new(pages: u64, flush: TlbFlush) -> Self {
        Self { pages, flush }
    }

    #[must_use]
    pub const fn pages(self) -> u64 {
        self.pages
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
pub struct ProtectRangeOutcome {
    pages: u64,
    flush: TlbFlush,
}

impl ProtectRangeOutcome {
    #[must_use]
    pub const fn new(pages: u64, flush: TlbFlush) -> Self {
        Self { pages, flush }
    }

    #[must_use]
    pub const fn pages(self) -> u64 {
        self.pages
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
pub struct PageTableAudit {
    total_tables: u64,
    used_tables: u64,
    reachable_tables: u64,
    mapped_pages: u64,
}

impl PageTableAudit {
    #[must_use]
    pub const fn new(
        total_tables: u64,
        used_tables: u64,
        reachable_tables: u64,
        mapped_pages: u64,
    ) -> Self {
        Self {
            total_tables,
            used_tables,
            reachable_tables,
            mapped_pages,
        }
    }

    #[must_use]
    pub const fn total_tables(self) -> u64 {
        self.total_tables
    }

    #[must_use]
    pub const fn used_tables(self) -> u64 {
        self.used_tables
    }

    #[must_use]
    pub const fn reachable_tables(self) -> u64 {
        self.reachable_tables
    }

    #[must_use]
    pub const fn mapped_pages(self) -> u64 {
        self.mapped_pages
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PageTableMappingSummary {
    pub(crate) total_pages: u64,
    pub(crate) kernel_pages: u64,
    pub(crate) user_pages: u64,
    pub(crate) writable_pages: u64,
    pub(crate) executable_pages: u64,
    pub(crate) global_pages: u64,
    pub(crate) device_pages: u64,
}

impl PageTableMappingSummary {
    #[must_use]
    pub const fn new(
        total_pages: u64,
        kernel_pages: u64,
        user_pages: u64,
        writable_pages: u64,
        executable_pages: u64,
        global_pages: u64,
        device_pages: u64,
    ) -> Self {
        Self {
            total_pages,
            kernel_pages,
            user_pages,
            writable_pages,
            executable_pages,
            global_pages,
            device_pages,
        }
    }

    #[must_use]
    pub const fn total_pages(self) -> u64 {
        self.total_pages
    }

    #[must_use]
    pub const fn kernel_pages(self) -> u64 {
        self.kernel_pages
    }

    #[must_use]
    pub const fn user_pages(self) -> u64 {
        self.user_pages
    }

    #[must_use]
    pub const fn writable_pages(self) -> u64 {
        self.writable_pages
    }

    #[must_use]
    pub const fn executable_pages(self) -> u64 {
        self.executable_pages
    }

    #[must_use]
    pub const fn global_pages(self) -> u64 {
        self.global_pages
    }

    #[must_use]
    pub const fn device_pages(self) -> u64 {
        self.device_pages
    }
}
