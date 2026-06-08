use crate::{
    ArchKind, BootInfo, CpuInfo, CpuTopology, FramebufferInfo, HhdmInfo, KernelImageInfo,
    MemoryMap, MemoryRegion, PlatformKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
        validate_kernel_image(metadata.kernel_image)?;

        Ok(Self {
            arch: metadata.arch,
            platform: metadata.platform,
            memory_map: MemoryMap::new(metadata.memory_regions),
            framebuffer: metadata.framebuffer,
            rsdp: metadata.rsdp,
            device_tree: metadata.device_tree,
            cpu_topology: CpuTopology::new(metadata.cpu_topology),
            kernel_image: metadata.kernel_image,
            hhdm: metadata.hhdm,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootInfoError {
    EmptyMemoryMap,
    InvalidMemoryRegion,
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

fn validate_kernel_image(image: KernelImageInfo) -> Result<(), BootInfoError> {
    if image.virt_end().get() <= image.virt_start().get() || image.phys_start().get() == 0 {
        return Err(BootInfoError::KernelImageEmpty);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use aesynx_abi::{PhysAddr, VirtAddr};

    use crate::{
        ArchKind, BootInfo, BootInfoError, BootMetadata, KernelImageInfo, MemoryRegion,
        MemoryRegionKind, PlatformKind,
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

        let summary = info.memory_map.summary();
        assert_eq!(summary.region_count, 2);
        assert_eq!(summary.usable_regions, 1);
        assert_eq!(summary.usable_bytes, 0x9000);
        assert_eq!(info.rsdp, Some(VirtAddr::new(0xffff800000007000)));
        Ok(())
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
    fn kernel_image_debug_redacts_addresses() {
        let info = KernelImageInfo::new(VirtAddr::new(1), VirtAddr::new(2), PhysAddr::new(3));
        let mut output = FixedBuf::default();

        assert_eq!(fmt::write(&mut output, format_args!("{info:?}")), Ok(()));
        assert_eq!(output.as_str(), "KernelImageInfo(redacted)");
    }

    #[derive(Default)]
    struct FixedBuf {
        bytes: [u8; 32],
        len: usize,
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
