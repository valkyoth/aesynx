#![no_std]
#![deny(unsafe_code)]

mod normalize;
mod types;

pub use normalize::{BootInfoError, BootMetadata};
pub use types::{
    ArchKind, BootInfo, CpuInfo, CpuTopology, FramebufferInfo, HhdmInfo, KernelImageInfo,
    MAX_EARLY_MEMORY_REGIONS, MemoryMap, MemoryRegion, MemoryRegionKind, MemorySummary,
    PlatformKind,
};
