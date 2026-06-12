#![no_std]
#![deny(unsafe_code)]

#[cfg(test)]
extern crate alloc;

mod normalize;
mod types;

#[cfg(test)]
mod fuzz;

pub use normalize::{BootInfoError, BootMetadata};
pub use types::{
    ArchKind, BootInfo, CpuInfo, CpuTopology, FRAME_SIZE, FramebufferInfo, HhdmInfo,
    KernelImageInfo, MAX_EARLY_MEMORY_REGIONS, MemoryAccountingError, MemoryMap, MemoryRegion,
    MemoryRegionKind, MemorySummary, PlatformKind,
};
