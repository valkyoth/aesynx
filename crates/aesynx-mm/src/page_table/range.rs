use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{FRAME_SIZE, GenericPageFlags};

use super::{MapRangeOutcome, PageTableError, PageTableMapper, TlbFlush, UnmapRangeOutcome};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn map_contiguous(
        &mut self,
        virt: VirtAddr,
        phys: PhysAddr,
        page_count: u64,
        flags: GenericPageFlags,
    ) -> Result<MapRangeOutcome, PageTableError> {
        validate_page_count(page_count)?;

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

    pub fn unmap_contiguous(
        &mut self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<UnmapRangeOutcome, PageTableError> {
        validate_page_count(page_count)?;

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

fn add_pages_to_virt(virt: VirtAddr, pages: u64) -> Result<VirtAddr, PageTableError> {
    let offset = pages
        .checked_mul(FRAME_SIZE)
        .ok_or(PageTableError::AddressOverflow)?;
    virt.get()
        .checked_add(offset)
        .map(VirtAddr::new)
        .ok_or(PageTableError::AddressOverflow)
}

fn add_pages_to_phys(phys: PhysAddr, pages: u64) -> Result<PhysAddr, PageTableError> {
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
    match (left, right) {
        (TlbFlush::AddressSpace, _) | (_, TlbFlush::AddressSpace) => TlbFlush::AddressSpace,
        (TlbFlush::None, flush) | (flush, TlbFlush::None) => flush,
        (TlbFlush::Page(left_page), TlbFlush::Page(right_page)) if left_page == right_page => {
            TlbFlush::Page(left_page)
        }
        (TlbFlush::Page(_), TlbFlush::Page(_)) => TlbFlush::AddressSpace,
    }
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
