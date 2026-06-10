use crate::{
    ArchKind, BootInfo, CpuInfo, CpuTopology, FramebufferInfo, HhdmInfo, KernelImageInfo,
    MemoryAccountingError, MemoryMap, MemoryRegion, PlatformKind, types::BootInfoParts,
};

const PAGE_SIZE: u64 = 4096;
const AARCH64_KERNEL_VMA_MIN: u64 = 0xffff_0000_0000_0000;
const X86_64_KERNEL_VMA_MIN: u64 = 0xffff_8000_0000_0000;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct BootMetadata<'a> {
    pub arch: ArchKind,
    pub platform: PlatformKind,
    pub memory_regions: &'a [MemoryRegion],
    pub framebuffer: Option<FramebufferInfo>,
    pub rsdp: Option<aesynx_abi::VirtAddr>,
    pub device_tree: Option<aesynx_abi::VirtAddr>,
    pub cpu_topology: &'a [CpuInfo],
    pub kernel_image: KernelImageInfo,
    pub hhdm: Option<HhdmInfo>,
}

impl<'a> BootInfo<'a> {
    pub fn normalize(metadata: BootMetadata<'a>) -> Result<Self, BootInfoError> {
        validate_memory_regions(metadata.memory_regions)?;
        MemoryMap::new(metadata.memory_regions)
            .summary()
            .map_err(BootInfoError::MemoryAccounting)?;
        validate_kernel_image(metadata.arch, metadata.kernel_image)?;

        Ok(Self::new(BootInfoParts {
            arch: metadata.arch,
            platform: metadata.platform,
            memory_map: MemoryMap::new(metadata.memory_regions),
            framebuffer: metadata.framebuffer,
            rsdp: metadata.rsdp,
            device_tree: metadata.device_tree,
            cpu_topology: CpuTopology::new(metadata.cpu_topology),
            kernel_image: metadata.kernel_image,
            hhdm: metadata.hhdm,
        }))
    }
}

impl core::fmt::Debug for BootMetadata<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootMetadata")
            .field("arch", &self.arch)
            .field("platform", &self.platform)
            .field("memory_regions.len", &self.memory_regions.len())
            .field("framebuffer_present", &self.framebuffer.is_some())
            .field("rsdp_present", &self.rsdp.is_some())
            .field("device_tree_present", &self.device_tree.is_some())
            .field("cpu_topology.len", &self.cpu_topology.len())
            .field("kernel_image", &self.kernel_image)
            .field("hhdm_present", &self.hhdm.is_some())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootInfoError {
    EmptyMemoryMap,
    InvalidMemoryRegion,
    MemoryAccounting(MemoryAccountingError),
    KernelImageEmpty,
}

fn validate_memory_regions(regions: &[MemoryRegion]) -> Result<(), BootInfoError> {
    if regions.is_empty() {
        return Err(BootInfoError::EmptyMemoryMap);
    }

    for region in regions {
        if region.len == 0 || region.end().is_none() {
            return Err(BootInfoError::InvalidMemoryRegion);
        }
    }

    Ok(())
}

fn validate_kernel_image(arch: ArchKind, image: KernelImageInfo) -> Result<(), BootInfoError> {
    let virt_start = image.virt_start().get();
    let virt_end = image.virt_end().get();
    let phys_start = image.phys_start().get();

    if virt_end <= virt_start {
        return Err(BootInfoError::KernelImageEmpty);
    }

    if phys_start == 0 || !phys_start.is_multiple_of(PAGE_SIZE) {
        return Err(BootInfoError::KernelImageEmpty);
    }

    if !virt_start.is_multiple_of(PAGE_SIZE) {
        return Err(BootInfoError::KernelImageEmpty);
    }

    let min_kernel_vma = match arch {
        ArchKind::Aarch64 => AARCH64_KERNEL_VMA_MIN,
        ArchKind::X86_64 => X86_64_KERNEL_VMA_MIN,
        ArchKind::Unknown => return Err(BootInfoError::KernelImageEmpty),
    };

    if virt_start < min_kernel_vma {
        return Err(BootInfoError::KernelImageEmpty);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use aesynx_abi::{PhysAddr, VirtAddr};

    use crate::{
        ArchKind, BootInfo, BootInfoError, BootMetadata, FramebufferInfo, HhdmInfo,
        KernelImageInfo, MemoryAccountingError, MemoryRegion, MemoryRegionKind, PlatformKind,
    };

    #[test]
    fn bootinfo_normalizes_synthetic_memory_map() -> Result<(), BootInfoError> {
        let regions = [
            MemoryRegion::new(PhysAddr::new(0x1000), 0x9000, MemoryRegionKind::Usable),
            MemoryRegion::new(PhysAddr::new(0x10000), 0x2000, MemoryRegionKind::Bootloader),
        ];

        let info = BootInfo::normalize(BootMetadata {
            arch: ArchKind::X86_64,
            platform: PlatformKind::Qemu,
            memory_regions: &regions,
            framebuffer: None,
            rsdp: Some(VirtAddr::new(0xffff800000007000)),
            device_tree: None,
            cpu_topology: &[],
            kernel_image: KernelImageInfo::new(
                VirtAddr::new(0xffffffff80000000),
                VirtAddr::new(0xffffffff80002000),
                PhysAddr::new(0x200000),
            ),
            hhdm: None,
        })?;

        assert_eq!(
            info.memory_map.summary(),
            Ok(crate::MemorySummary {
                region_count: 2,
                total_bytes: 0xb000,
                total_frames: 11,
                usable_regions: 1,
                usable_bytes: 0x9000,
                usable_frames: 9,
                reserved_regions: 1,
                reserved_bytes: 0x2000,
                reserved_frames: 2,
                kernel_bytes: 0,
                bootloader_bytes: 0x2000,
                framebuffer_bytes: 0,
                acpi_bytes: 0,
                bad_bytes: 0,
            })
        );
        assert!(info.rsdp_present());
        Ok(())
    }

    #[test]
    fn memory_summary_accounts_region_kinds_and_full_frames() {
        let regions = [
            MemoryRegion::new(PhysAddr::new(0x1001), 0x2fff, MemoryRegionKind::Usable),
            MemoryRegion::new(PhysAddr::new(0x5000), 0x2000, MemoryRegionKind::Kernel),
            MemoryRegion::new(PhysAddr::new(0x7000), 0x1000, MemoryRegionKind::Framebuffer),
            MemoryRegion::new(PhysAddr::new(0x8000), 0x1000, MemoryRegionKind::Acpi),
            MemoryRegion::new(PhysAddr::new(0x9000), 0x1000, MemoryRegionKind::Bad),
        ];

        let summary = crate::MemoryMap::new(&regions).summary();

        assert_eq!(
            summary,
            Ok(crate::MemorySummary {
                region_count: 5,
                total_bytes: 0x7fff,
                total_frames: 7,
                usable_regions: 1,
                usable_bytes: 0x2fff,
                usable_frames: 2,
                reserved_regions: 4,
                reserved_bytes: 0x5000,
                reserved_frames: 5,
                kernel_bytes: 0x2000,
                bootloader_bytes: 0,
                framebuffer_bytes: 0x1000,
                acpi_bytes: 0x1000,
                bad_bytes: 0x1000,
            })
        );
    }

    #[test]
    fn bootinfo_rejects_empty_memory_map() {
        let result = BootInfo::normalize(BootMetadata {
            arch: ArchKind::X86_64,
            platform: PlatformKind::Qemu,
            memory_regions: &[],
            framebuffer: None,
            rsdp: None,
            device_tree: None,
            cpu_topology: &[],
            kernel_image: KernelImageInfo::new(
                VirtAddr::new(0xffffffff80000000),
                VirtAddr::new(0xffffffff80002000),
                PhysAddr::new(0x200000),
            ),
            hhdm: None,
        });

        assert_eq!(result, Err(BootInfoError::EmptyMemoryMap));
    }

    #[test]
    fn bootinfo_rejects_memory_accounting_overflow() {
        let regions = [
            MemoryRegion::new(PhysAddr::new(0), u64::MAX - 1, MemoryRegionKind::Usable),
            MemoryRegion::new(PhysAddr::new(0), 2, MemoryRegionKind::Usable),
        ];
        let result = BootInfo::normalize(BootMetadata {
            arch: ArchKind::X86_64,
            platform: PlatformKind::Qemu,
            memory_regions: &regions,
            framebuffer: None,
            rsdp: None,
            device_tree: None,
            cpu_topology: &[],
            kernel_image: KernelImageInfo::new(
                VirtAddr::new(0xffffffff80000000),
                VirtAddr::new(0xffffffff80002000),
                PhysAddr::new(0x200000),
            ),
            hhdm: None,
        });

        assert_eq!(
            result,
            Err(BootInfoError::MemoryAccounting(
                MemoryAccountingError::Overflow
            ))
        );
    }

    #[test]
    fn kernel_image_debug_redacts_addresses() {
        let info = KernelImageInfo::new(VirtAddr::new(1), VirtAddr::new(2), PhysAddr::new(3));
        let mut output = FixedBuf::default();

        assert_eq!(fmt::write(&mut output, format_args!("{info:?}")), Ok(()));
        assert_eq!(output.as_str(), "KernelImageInfo(redacted)");
    }

    #[test]
    fn bootinfo_debug_redacts_handoff_addresses() -> Result<(), BootInfoError> {
        let regions = [
            MemoryRegion::new(PhysAddr::new(0x1000), 0x9000, MemoryRegionKind::Usable),
            MemoryRegion::new(PhysAddr::new(0x200000), 0x2000, MemoryRegionKind::Kernel),
        ];
        let metadata = BootMetadata {
            arch: ArchKind::X86_64,
            platform: PlatformKind::Qemu,
            memory_regions: &regions,
            framebuffer: Some(FramebufferInfo::new(
                VirtAddr::new(0xffff80000000b800),
                80,
                25,
                80,
            )),
            rsdp: Some(VirtAddr::new(0xffff800000007000)),
            device_tree: Some(VirtAddr::new(0xffff800000008000)),
            cpu_topology: &[],
            kernel_image: KernelImageInfo::new(
                VirtAddr::new(0xffffffff80000000),
                VirtAddr::new(0xffffffff80002000),
                PhysAddr::new(0x200000),
            ),
            hhdm: Some(HhdmInfo::new(VirtAddr::new(0xffff800000000000))),
        };
        let info = BootInfo::normalize(metadata)?;
        let mut output = FixedBuf::default();

        assert_eq!(fmt::write(&mut output, format_args!("{info:?}")), Ok(()));

        let debug = output.as_str();
        assert!(debug.contains("rsdp_present: true"));
        assert!(debug.contains("hhdm_present: true"));
        assert!(!debug.contains("ffff"));
        assert!(!debug.contains("200000"));
        Ok(())
    }

    #[test]
    fn memory_region_debug_redacts_physical_start() {
        let region = MemoryRegion::new(PhysAddr::new(0x200000), 0x2000, MemoryRegionKind::Kernel);
        let mut output = FixedBuf::default();

        assert_eq!(fmt::write(&mut output, format_args!("{region:?}")), Ok(()));

        let debug = output.as_str();
        assert!(debug.contains("start_present: true"));
        assert!(debug.contains("kind: Kernel"));
        assert!(!debug.contains("200000"));
    }

    #[test]
    fn bootmetadata_debug_redacts_handoff_addresses() {
        let regions = [MemoryRegion::new(
            PhysAddr::new(0x1000),
            0x9000,
            MemoryRegionKind::Usable,
        )];
        let metadata = BootMetadata {
            arch: ArchKind::X86_64,
            platform: PlatformKind::Qemu,
            memory_regions: &regions,
            framebuffer: Some(FramebufferInfo::new(
                VirtAddr::new(0xffff80000000b800),
                80,
                25,
                80,
            )),
            rsdp: Some(VirtAddr::new(0xffff800000007000)),
            device_tree: None,
            cpu_topology: &[],
            kernel_image: KernelImageInfo::new(
                VirtAddr::new(0xffffffff80000000),
                VirtAddr::new(0xffffffff80002000),
                PhysAddr::new(0x200000),
            ),
            hhdm: Some(HhdmInfo::new(VirtAddr::new(0xffff800000000000))),
        };
        let mut output = FixedBuf::default();

        assert_eq!(
            fmt::write(&mut output, format_args!("{metadata:?}")),
            Ok(())
        );

        let debug = output.as_str();
        assert!(debug.contains("rsdp_present: true"));
        assert!(debug.contains("KernelImageInfo(redacted)"));
        assert!(!debug.contains("ffff"));
    }

    #[test]
    fn bootinfo_rejects_misaligned_kernel_image() {
        let regions = [MemoryRegion::new(
            PhysAddr::new(0x1000),
            0x9000,
            MemoryRegionKind::Usable,
        )];
        let result = BootInfo::normalize(BootMetadata {
            arch: ArchKind::X86_64,
            platform: PlatformKind::Qemu,
            memory_regions: &regions,
            framebuffer: None,
            rsdp: None,
            device_tree: None,
            cpu_topology: &[],
            kernel_image: KernelImageInfo::new(
                VirtAddr::new(0xffffffff80000001),
                VirtAddr::new(0xffffffff80002000),
                PhysAddr::new(0x200000),
            ),
            hhdm: None,
        });

        assert_eq!(result, Err(BootInfoError::KernelImageEmpty));
    }

    #[test]
    fn bootinfo_rejects_user_half_x86_64_kernel_image() {
        let regions = [MemoryRegion::new(
            PhysAddr::new(0x1000),
            0x9000,
            MemoryRegionKind::Usable,
        )];
        let result = BootInfo::normalize(BootMetadata {
            arch: ArchKind::X86_64,
            platform: PlatformKind::Qemu,
            memory_regions: &regions,
            framebuffer: None,
            rsdp: None,
            device_tree: None,
            cpu_topology: &[],
            kernel_image: KernelImageInfo::new(
                VirtAddr::new(0x400000),
                VirtAddr::new(0x402000),
                PhysAddr::new(0x200000),
            ),
            hhdm: None,
        });

        assert_eq!(result, Err(BootInfoError::KernelImageEmpty));
    }

    #[test]
    fn bootinfo_rejects_user_half_aarch64_kernel_image() {
        let regions = [MemoryRegion::new(
            PhysAddr::new(0x1000),
            0x9000,
            MemoryRegionKind::Usable,
        )];
        let result = BootInfo::normalize(BootMetadata {
            arch: ArchKind::Aarch64,
            platform: PlatformKind::Qemu,
            memory_regions: &regions,
            framebuffer: None,
            rsdp: None,
            device_tree: None,
            cpu_topology: &[],
            kernel_image: KernelImageInfo::new(
                VirtAddr::new(0x400000),
                VirtAddr::new(0x402000),
                PhysAddr::new(0x200000),
            ),
            hhdm: None,
        });

        assert_eq!(result, Err(BootInfoError::KernelImageEmpty));
    }

    struct FixedBuf {
        bytes: [u8; 512],
        len: usize,
    }

    impl Default for FixedBuf {
        fn default() -> Self {
            Self {
                bytes: [0; 512],
                len: 0,
            }
        }
    }

    impl FixedBuf {
        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.bytes[..self.len]).unwrap_or_default()
        }
    }

    impl fmt::Write for FixedBuf {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            if self.len + value.len() > self.bytes.len() {
                return Err(fmt::Error);
            }

            let end = self.len + value.len();
            self.bytes[self.len..end].copy_from_slice(value.as_bytes());
            self.len = end;
            Ok(())
        }
    }
}
