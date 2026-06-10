use aesynx_abi::{PhysAddr, VirtAddr};

use crate::FRAME_SIZE;

use super::{PAGE_TABLE_LEVELS, PageTableError};

pub(super) const PAGE_OFFSET_MASK: u64 = FRAME_SIZE - 1;

const CANONICAL_LOW_END: u64 = 0x0000_7fff_ffff_ffff;
const CANONICAL_HIGH_START: u64 = 0xffff_8000_0000_0000;
const MAX_PHYSICAL_ADDR: u64 = 0x000f_ffff_ffff_ffff;
const LEVEL_SHIFTS: [u64; PAGE_TABLE_LEVELS] = [39, 30, 21, 12];

pub(super) fn validate_virt_page(virt: VirtAddr) -> Result<(), PageTableError> {
    if !is_canonical(virt.get()) {
        return Err(PageTableError::InvalidVirtualAddress);
    }
    if virt.get() & PAGE_OFFSET_MASK != 0 {
        return Err(PageTableError::UnalignedVirtualAddress);
    }
    Ok(())
}

pub(super) fn validate_phys(phys: PhysAddr) -> Result<(), PageTableError> {
    if phys.get() > MAX_PHYSICAL_ADDR {
        return Err(PageTableError::InvalidPhysicalAddress);
    }
    if phys.get() & PAGE_OFFSET_MASK != 0 {
        return Err(PageTableError::UnalignedPhysicalAddress);
    }
    Ok(())
}

pub(super) const fn is_canonical(value: u64) -> bool {
    value <= CANONICAL_LOW_END || value >= CANONICAL_HIGH_START
}

pub(super) fn page_indices(virt: VirtAddr) -> [usize; PAGE_TABLE_LEVELS] {
    [
        ((virt.get() >> LEVEL_SHIFTS[0]) & 0x1ff) as usize,
        ((virt.get() >> LEVEL_SHIFTS[1]) & 0x1ff) as usize,
        ((virt.get() >> LEVEL_SHIFTS[2]) & 0x1ff) as usize,
        ((virt.get() >> LEVEL_SHIFTS[3]) & 0x1ff) as usize,
    ]
}
