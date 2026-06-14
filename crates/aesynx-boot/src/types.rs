use core::fmt;

use aesynx_abi::{CpuHardwareId, PhysAddr, VirtAddr};

pub const MAX_EARLY_MEMORY_REGIONS: usize = 64;
pub const FRAME_SIZE: u64 = 4096;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct BootInfo<'a> {
    pub arch: ArchKind,
    pub platform: PlatformKind,
    pub memory_map: MemoryMap<'a>,
    framebuffer: Option<FramebufferInfo>,
    rsdp: Option<VirtAddr>,
    device_tree: Option<VirtAddr>,
    pub cpu_topology: CpuTopology<'a>,
    pub kernel_image: KernelImageInfo,
    hhdm: Option<HhdmInfo>,
}

impl<'a> BootInfo<'a> {
    #[must_use]
    pub(crate) const fn new(parts: BootInfoParts<'a>) -> Self {
        Self {
            arch: parts.arch,
            platform: parts.platform,
            memory_map: parts.memory_map,
            framebuffer: parts.framebuffer,
            rsdp: parts.rsdp,
            device_tree: parts.device_tree,
            cpu_topology: parts.cpu_topology,
            kernel_image: parts.kernel_image,
            hhdm: parts.hhdm,
        }
    }

    #[must_use]
    pub const fn framebuffer_present(self) -> bool {
        self.framebuffer.is_some()
    }

    #[must_use]
    pub const fn framebuffer_base(self) -> Option<VirtAddr> {
        match self.framebuffer {
            Some(framebuffer) => Some(framebuffer.base),
            None => None,
        }
    }

    #[must_use]
    pub const fn rsdp_present(self) -> bool {
        self.rsdp.is_some()
    }

    #[must_use]
    pub const fn device_tree_present(self) -> bool {
        self.device_tree.is_some()
    }

    #[must_use]
    pub const fn hhdm_present(self) -> bool {
        self.hhdm.is_some()
    }

    #[must_use]
    pub const fn hhdm_offset(self) -> Option<VirtAddr> {
        match self.hhdm {
            Some(hhdm) => Some(hhdm.offset),
            None => None,
        }
    }
}

pub(crate) struct BootInfoParts<'a> {
    pub(crate) arch: ArchKind,
    pub(crate) platform: PlatformKind,
    pub(crate) memory_map: MemoryMap<'a>,
    pub(crate) framebuffer: Option<FramebufferInfo>,
    pub(crate) rsdp: Option<VirtAddr>,
    pub(crate) device_tree: Option<VirtAddr>,
    pub(crate) cpu_topology: CpuTopology<'a>,
    pub(crate) kernel_image: KernelImageInfo,
    pub(crate) hhdm: Option<HhdmInfo>,
}

impl fmt::Debug for BootInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BootInfo")
            .field("arch", &self.arch)
            .field("platform", &self.platform)
            .field("memory_map.len", &self.memory_map.len())
            .field("framebuffer_present", &self.framebuffer_present())
            .field("rsdp_present", &self.rsdp_present())
            .field("device_tree_present", &self.device_tree_present())
            .field("cpu_topology.len", &self.cpu_topology.len())
            .field("kernel_image", &self.kernel_image)
            .field("hhdm_present", &self.hhdm_present())
            .finish()
    }
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
pub struct MemoryMap<'a> {
    regions: &'a [MemoryRegion],
}

impl<'a> MemoryMap<'a> {
    #[must_use]
    pub const fn new(regions: &'a [MemoryRegion]) -> Self {
        Self { regions }
    }

    #[must_use]
    pub const fn regions(self) -> &'a [MemoryRegion] {
        self.regions
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.regions.len()
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.regions.is_empty()
    }

    pub fn summary(self) -> Result<MemorySummary, MemoryAccountingError> {
        let mut usable_regions = 0usize;
        let mut usable_bytes = 0u64;
        let mut reserved_regions = 0usize;
        let mut reserved_bytes = 0u64;
        let mut kernel_bytes = 0u64;
        let mut bootloader_bytes = 0u64;
        let mut framebuffer_bytes = 0u64;
        let mut acpi_bytes = 0u64;
        let mut bad_bytes = 0u64;
        let mut total_bytes = 0u64;
        let mut usable_frames = 0u64;
        let mut reserved_frames = 0u64;
        let mut total_frames = 0u64;

        for region in self.regions {
            let frames = region.full_frame_count()?;
            total_bytes = checked_add(total_bytes, region.len)?;
            total_frames = checked_add(total_frames, frames)?;
            match region.kind {
                MemoryRegionKind::Usable => {
                    usable_regions += 1;
                    usable_bytes = checked_add(usable_bytes, region.len)?;
                    usable_frames = checked_add(usable_frames, frames)?;
                }
                MemoryRegionKind::Reserved => {
                    reserved_regions += 1;
                    reserved_bytes = checked_add(reserved_bytes, region.len)?;
                    reserved_frames = checked_add(reserved_frames, frames)?;
                }
                MemoryRegionKind::Kernel => {
                    reserved_regions += 1;
                    reserved_bytes = checked_add(reserved_bytes, region.len)?;
                    kernel_bytes = checked_add(kernel_bytes, region.len)?;
                    reserved_frames = checked_add(reserved_frames, frames)?;
                }
                MemoryRegionKind::Bootloader => {
                    reserved_regions += 1;
                    reserved_bytes = checked_add(reserved_bytes, region.len)?;
                    bootloader_bytes = checked_add(bootloader_bytes, region.len)?;
                    reserved_frames = checked_add(reserved_frames, frames)?;
                }
                MemoryRegionKind::Framebuffer => {
                    reserved_regions += 1;
                    reserved_bytes = checked_add(reserved_bytes, region.len)?;
                    framebuffer_bytes = checked_add(framebuffer_bytes, region.len)?;
                    reserved_frames = checked_add(reserved_frames, frames)?;
                }
                MemoryRegionKind::Acpi => {
                    reserved_regions += 1;
                    reserved_bytes = checked_add(reserved_bytes, region.len)?;
                    acpi_bytes = checked_add(acpi_bytes, region.len)?;
                    reserved_frames = checked_add(reserved_frames, frames)?;
                }
                MemoryRegionKind::Bad => {
                    reserved_regions += 1;
                    reserved_bytes = checked_add(reserved_bytes, region.len)?;
                    bad_bytes = checked_add(bad_bytes, region.len)?;
                    reserved_frames = checked_add(reserved_frames, frames)?;
                }
            }
        }

        Ok(MemorySummary {
            region_count: self.regions.len(),
            total_bytes,
            total_frames,
            usable_regions,
            usable_bytes,
            usable_frames,
            reserved_regions,
            reserved_bytes,
            reserved_frames,
            kernel_bytes,
            bootloader_bytes,
            framebuffer_bytes,
            acpi_bytes,
            bad_bytes,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemorySummary {
    pub region_count: usize,
    pub total_bytes: u64,
    pub total_frames: u64,
    pub usable_regions: usize,
    pub usable_bytes: u64,
    pub usable_frames: u64,
    pub reserved_regions: usize,
    pub reserved_bytes: u64,
    pub reserved_frames: u64,
    pub kernel_bytes: u64,
    pub bootloader_bytes: u64,
    pub framebuffer_bytes: u64,
    pub acpi_bytes: u64,
    pub bad_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryAccountingError {
    Overflow,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct MemoryRegion {
    pub(crate) start: PhysAddr,
    pub len: u64,
    pub kind: MemoryRegionKind,
}

impl MemoryRegion {
    pub const EMPTY: Self = Self {
        start: PhysAddr::new(0),
        len: 0,
        kind: MemoryRegionKind::Reserved,
    };

    #[must_use]
    pub const fn new(start: PhysAddr, len: u64, kind: MemoryRegionKind) -> Self {
        Self { start, len, kind }
    }

    #[must_use]
    pub const fn start(self) -> PhysAddr {
        self.start
    }

    #[must_use]
    pub const fn start_present(self) -> bool {
        self.start.get() != 0
    }

    #[must_use]
    pub const fn end(self) -> Option<PhysAddr> {
        match self.start.get().checked_add(self.len) {
            Some(end) => Some(PhysAddr::new(end)),
            None => None,
        }
    }

    pub(crate) fn full_frame_count(self) -> Result<u64, MemoryAccountingError> {
        let start = align_up(self.start.get())?;
        let Some(end) = self.end() else {
            return Err(MemoryAccountingError::Overflow);
        };
        let end = align_down(end.get());
        if end <= start {
            return Ok(0);
        }

        Ok((end - start) / FRAME_SIZE)
    }
}

impl fmt::Debug for MemoryRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryRegion")
            .field("start_present", &self.start_present())
            .field("len", &self.len)
            .field("kind", &self.kind)
            .finish()
    }
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

const fn checked_add(left: u64, right: u64) -> Result<u64, MemoryAccountingError> {
    match left.checked_add(right) {
        Some(value) => Ok(value),
        None => Err(MemoryAccountingError::Overflow),
    }
}

const fn align_up(value: u64) -> Result<u64, MemoryAccountingError> {
    let mask = FRAME_SIZE - 1;
    match value.checked_add(mask) {
        Some(value) => Ok(value & !mask),
        None => Err(MemoryAccountingError::Overflow),
    }
}

const fn align_down(value: u64) -> u64 {
    value & !(FRAME_SIZE - 1)
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FramebufferInfo {
    base: VirtAddr,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

impl FramebufferInfo {
    #[must_use]
    pub const fn new(base: VirtAddr, width: u32, height: u32, stride: u32) -> Self {
        Self {
            base,
            width,
            height,
            stride,
        }
    }
}

impl fmt::Debug for FramebufferInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FramebufferInfo")
            .field("base_present", &(self.base.get() != 0))
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .finish()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HhdmInfo {
    offset: VirtAddr,
}

impl HhdmInfo {
    #[must_use]
    pub const fn new(offset: VirtAddr) -> Self {
        Self { offset }
    }
}

impl fmt::Debug for HhdmInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HhdmInfo")
            .field("offset_present", &(self.offset.get() != 0))
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuTopology<'a> {
    cpus: &'a [CpuInfo],
}

impl<'a> CpuTopology<'a> {
    #[must_use]
    pub const fn new(cpus: &'a [CpuInfo]) -> Self {
        Self { cpus }
    }

    #[must_use]
    pub const fn cpus(self) -> &'a [CpuInfo] {
        self.cpus
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.cpus.len()
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.cpus.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuInfo {
    pub hardware_id: CpuHardwareId,
    pub enabled: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct KernelImageInfo {
    virt_start: VirtAddr,
    virt_end: VirtAddr,
    phys_start: PhysAddr,
}

impl KernelImageInfo {
    /// Constructs a kernel image descriptor from bootloader-provided addresses.
    ///
    /// # Security note
    ///
    /// The address fields are intentionally opaque outside this crate. Widening
    /// the visibility of `virt_start`, `virt_end`, or `phys_start` would weaken
    /// the KASLR protection used by `BootInfo` and boot metadata redaction.
    #[must_use]
    pub const fn new(virt_start: VirtAddr, virt_end: VirtAddr, phys_start: PhysAddr) -> Self {
        Self {
            virt_start,
            virt_end,
            phys_start,
        }
    }

    #[must_use]
    pub fn phys_for_virt(self, virt: VirtAddr) -> Option<PhysAddr> {
        let virt = virt.get();
        if virt < self.virt_start.get() || virt >= self.virt_end.get() {
            return None;
        }

        virt.checked_sub(self.virt_start.get())
            .and_then(|offset| self.phys_start.get().checked_add(offset))
            .map(PhysAddr::new)
    }

    #[must_use]
    pub(crate) const fn virt_start(self) -> VirtAddr {
        self.virt_start
    }

    #[must_use]
    pub(crate) const fn virt_end(self) -> VirtAddr {
        self.virt_end
    }

    #[must_use]
    pub(crate) const fn phys_start(self) -> PhysAddr {
        self.phys_start
    }
}

impl fmt::Debug for KernelImageInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("KernelImageInfo(redacted)")
    }
}

#[cfg(test)]
mod tests;
