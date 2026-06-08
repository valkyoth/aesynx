use aesynx_abi::{CpuHardwareId, PhysAddr, VirtAddr};

pub const MAX_EARLY_MEMORY_REGIONS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootInfo<'a> {
    pub arch: ArchKind,
    pub platform: PlatformKind,
    pub memory_map: MemoryMap<'a>,
    pub framebuffer: Option<FramebufferInfo>,
    pub rsdp: Option<VirtAddr>,
    pub device_tree: Option<VirtAddr>,
    pub cpu_topology: CpuTopology<'a>,
    pub kernel_image: KernelImageInfo,
    pub hhdm: Option<HhdmInfo>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRegion {
    pub start: PhysAddr,
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
    pub const fn end(self) -> Option<PhysAddr> {
        match self.start.get().checked_add(self.len) {
            Some(end) => Some(PhysAddr::new(end)),
            None => None,
        }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FramebufferInfo {
    pub base: VirtAddr,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HhdmInfo {
    pub offset: VirtAddr,
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

impl core::fmt::Debug for KernelImageInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("KernelImageInfo(redacted)")
    }
}
