use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{FRAME_SIZE, GenericPageFlags};

use super::address::{validate_phys, validate_virt_page};
use super::{
    MapRangeOutcome, PageMapping, PageRangeMapping, PageTableError, PageTableMapper,
    ProtectRangeOutcome, TlbFlush, UnmapRangeOutcome, X86_64PageTableEntry,
};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn map_contiguous(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        page_count: u64,
        flags: GenericPageFlags,
    ) -> Result<MapRangeOutcome, PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_phys_range(phys, page_count)?;
        validate_flags(flags)?;
        validate_range_walk::<TABLES>(page_count)?;

        let mut candidate = *self;
        let mut offset = 0u64;
        while offset < page_count {
            candidate.map_page(
                add_pages_to_virt(virt, offset)?,
                add_pages_to_phys(phys, offset)?,
                flags,
            )?;
            offset += 1;
        }

        *self = candidate;
        Ok(MapRangeOutcome::new(
            page_count,
            flush_for_range(virt, page_count),
        ))
    }

    pub fn mapping_for_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<PageRangeMapping, PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let first = self.mapping_for_address(virt)?;
        let flags = first.flags();
        let mut offset = 1u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if mapping.phys() != add_pages_to_phys(first.phys(), offset)?
                || mapping.flags() != flags
            {
                return Err(PageTableError::NonContiguousRange);
            }
            offset += 1;
        }

        Ok(PageRangeMapping::new(first.phys(), page_count, flags))
    }

    pub fn ensure_unmapped_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            match self.mapping_for_address(add_pages_to_virt(virt, offset)?) {
                Ok(_mapping) => return Err(PageTableError::AlreadyMapped),
                Err(PageTableError::NotMapped) => {}
                Err(error) => return Err(error),
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn ensure_mapped_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            match self.mapping_for_address(add_pages_to_virt(virt, offset)?) {
                Ok(_mapping) => {}
                Err(PageTableError::NotMapped) => return Err(PageTableError::NotMapped),
                Err(error) => return Err(error),
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn ensure_contiguous_flags(
        &self,
        virt: VirtAddr,
        page_count: u64,
        flags: GenericPageFlags,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_flags(flags)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if mapping.flags() != flags {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn protect_contiguous(
        &mut self,
        virt: VirtAddr,
        page_count: u64,
        flags: GenericPageFlags,
    ) -> Result<ProtectRangeOutcome, PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_flags(flags)?;
        validate_range_walk::<TABLES>(page_count)?;

        let mut candidate = *self;
        let mut flush = TlbFlush::None;
        let mut offset = 0u64;
        while offset < page_count {
            let outcome = candidate.protect_page(add_pages_to_virt(virt, offset)?, flags)?;
            flush = combine_flushes(flush, outcome.flush());
            offset += 1;
        }

        *self = candidate;
        Ok(ProtectRangeOutcome::new(
            page_count,
            collapse_range_flush(virt, page_count, flush),
        ))
    }

    pub fn unmap_contiguous(
        &mut self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<UnmapRangeOutcome, PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;

        let mut candidate = *self;
        let mut flush = TlbFlush::None;
        let mut offset = 0u64;
        while offset < page_count {
            let outcome = candidate.unmap_page(add_pages_to_virt(virt, offset)?)?;
            flush = combine_flushes(flush, outcome.flush());
            offset += 1;
        }

        *self = candidate;
        Ok(UnmapRangeOutcome::new(
            page_count,
            collapse_range_flush(virt, page_count, flush),
        ))
    }
}

fn validate_page_count(page_count: u64) -> Result<(), PageTableError> {
    if page_count == 0 {
        return Err(PageTableError::InvalidPageCount);
    }
    Ok(())
}

pub(super) fn validate_virt_range(virt: VirtAddr, page_count: u64) -> Result<(), PageTableError> {
    validate_page_count(page_count)?;
    validate_virt_page(virt)?;

    let last = add_pages_to_virt(virt, page_count - 1)?;
    validate_virt_page(last)?;
    if canonical_sign_bit(virt) != canonical_sign_bit(last) {
        return Err(PageTableError::InvalidVirtualAddress);
    }

    Ok(())
}

fn validate_phys_range(phys: PhysAddr, page_count: u64) -> Result<(), PageTableError> {
    validate_page_count(page_count)?;
    validate_phys(phys)?;

    let last = add_pages_to_phys(phys, page_count - 1)?;
    validate_phys(last)
}

fn validate_flags(flags: GenericPageFlags) -> Result<(), PageTableError> {
    X86_64PageTableEntry::from_mapping(PageMapping::new(PhysAddr::new(0), flags))?;
    Ok(())
}

pub(super) fn validate_virtual_space(
    virt: VirtAddr,
    page_count: u64,
    space: VirtualSpace,
) -> Result<(), PageTableError> {
    validate_page_count(page_count)?;
    let last = add_pages_to_virt(virt, page_count - 1)?;
    if virtual_space(virt) != space || virtual_space(last) != space {
        return Err(PageTableError::UnexpectedVirtualAddressSpace);
    }
    Ok(())
}

pub(super) fn validate_range_walk<const TABLES: usize>(
    page_count: u64,
) -> Result<(), PageTableError> {
    validate_page_count(page_count)?;
    let table_walk_pages = (TABLES as u64)
        .checked_mul(super::PAGE_TABLE_ENTRIES as u64)
        .ok_or(PageTableError::AddressOverflow)?;
    let max_pages = if table_walk_pages > super::MAPPED_FRAME_INDEX_ENTRIES as u64 {
        super::MAPPED_FRAME_INDEX_ENTRIES as u64
    } else {
        table_walk_pages
    };
    if page_count > max_pages {
        return Err(PageTableError::RangeTooLarge);
    }
    Ok(())
}

fn canonical_sign_bit(virt: VirtAddr) -> u64 {
    (virt.get() >> 47) & 1
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum VirtualSpace {
    User,
    Kernel,
}

fn virtual_space(virt: VirtAddr) -> VirtualSpace {
    if canonical_sign_bit(virt) == 0 {
        VirtualSpace::User
    } else {
        VirtualSpace::Kernel
    }
}

pub(super) fn add_pages_to_virt(virt: VirtAddr, pages: u64) -> Result<VirtAddr, PageTableError> {
    let offset = pages
        .checked_mul(FRAME_SIZE)
        .ok_or(PageTableError::AddressOverflow)?;
    virt.get()
        .checked_add(offset)
        .map(VirtAddr::new)
        .ok_or(PageTableError::AddressOverflow)
}

pub(super) fn add_pages_to_phys(phys: PhysAddr, pages: u64) -> Result<PhysAddr, PageTableError> {
    let offset = pages
        .checked_mul(FRAME_SIZE)
        .ok_or(PageTableError::AddressOverflow)?;
    phys.get()
        .checked_add(offset)
        .map(PhysAddr::new)
        .ok_or(PageTableError::AddressOverflow)
}

fn flush_for_range(virt: VirtAddr, page_count: u64) -> TlbFlush {
    if page_count == 1 {
        TlbFlush::Page(virt)
    } else {
        TlbFlush::AddressSpace
    }
}

fn combine_flushes(left: TlbFlush, right: TlbFlush) -> TlbFlush {
    left.merge(right)
}

fn collapse_range_flush(virt: VirtAddr, page_count: u64, flush: TlbFlush) -> TlbFlush {
    if page_count == 1 {
        flush
    } else if flush == TlbFlush::None {
        TlbFlush::None
    } else {
        flush_for_range(virt, page_count)
    }
}
