#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CpuHardwareId, PhysAddr, VirtAddr};

#[derive(Debug, Eq, PartialEq)]
pub struct BootInfo<'a> {
    pub arch: ArchKind,
    pub platform: PlatformKind,
    pub memory_map: &'a [MemoryRegion],
    pub framebuffer: Option<FramebufferInfo>,
    pub rsdp: Option<PhysAddr>,
    pub device_tree: Option<PhysAddr>,
    pub cpu_topology: &'a [CpuInfo],
    pub kernel_image: KernelImageInfo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchKind {
    X86_64,
    Aarch64,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformKind {
    Qemu,
    Uefi,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRegion {
    pub start: PhysAddr,
    pub len: u64,
    pub kind: MemoryRegionKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
    Kernel,
    Bootloader,
    Framebuffer,
    Acpi,
    Bad,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FramebufferInfo {
    pub base: PhysAddr,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuInfo {
    pub hardware_id: CpuHardwareId,
    pub enabled: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct KernelImageInfo {
    /// KASLR-sensitive virtual start address. Do not log or expose outside the boot path.
    pub virt_start: VirtAddr,
    /// KASLR-sensitive virtual end address. Do not log or expose outside the boot path.
    pub virt_end: VirtAddr,
    /// KASLR-sensitive physical start address. Do not log or expose outside the boot path.
    pub phys_start: PhysAddr,
}
