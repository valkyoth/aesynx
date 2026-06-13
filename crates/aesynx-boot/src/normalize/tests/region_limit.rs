use aesynx_abi::PhysAddr;

use crate::{BootInfo, BootInfoError, MAX_EARLY_MEMORY_REGIONS, MemoryRegion, MemoryRegionKind};

use super::qemu_metadata;

#[test]
fn bootinfo_rejects_too_many_memory_regions_before_overlap_scan() {
    let mut regions = [MemoryRegion::EMPTY; MAX_EARLY_MEMORY_REGIONS + 1];
    let mut index = 0usize;
    while index < regions.len() {
        regions[index] = MemoryRegion::new(
            PhysAddr::new(0x1000 + (index as u64 * 0x2000)),
            0x1000,
            MemoryRegionKind::Usable,
        );
        index += 1;
    }

    let result = BootInfo::normalize(qemu_metadata(&regions));

    assert_eq!(result, Err(BootInfoError::InvalidMemoryRegion));
}
