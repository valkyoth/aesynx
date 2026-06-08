use core::fmt;

use aesynx_abi::{CpuHardwareId, PhysAddr, VirtAddr};

pub const MAX_EARLY_MEMORY_REGIONS: usize = 64;

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

    #[must_use]
    pub fn summary(self) -> MemorySummary {
        let mut usable_regions = 0usize;
        let mut usable_bytes = 0u64;

        for region in self.regions {
            if region.kind == MemoryRegionKind::Usable {
                usable_regions += 1;
                usable_bytes = usable_bytes.saturating_add(region.len);
            }
        }

        MemorySummary {
            region_count: self.regions.len(),
            usable_regions,
            usable_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemorySummary {
    pub region_count: usize,
    pub usable_regions: usize,
    pub usable_bytes: u64,
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
    pub const fn start_present(self) -> bool {
        self.start.get() != 0
    }

    #[must_use]
    pub(crate) const fn end(self) -> Option<PhysAddr> {
        match self.start.get().checked_add(self.len) {
            Some(end) => Some(PhysAddr::new(end)),
            None => None,
        }
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
