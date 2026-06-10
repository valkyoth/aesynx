#![no_std]
#![deny(unsafe_code)]

mod normalize;
mod types;

pub use normalize::{BootInfoError, BootMetadata};
pub use types::{
    ArchKind, BootInfo, CpuInfo, CpuTopology, FRAME_SIZE, FramebufferInfo, HhdmInfo,
    KernelImageInfo, MAX_EARLY_MEMORY_REGIONS, MemoryAccountingError, MemoryMap, MemoryRegion,
    MemoryRegionKind, MemorySummary, PlatformKind,
};
