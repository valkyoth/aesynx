#![no_std]
#![deny(unsafe_code)]

pub mod diagnostics;

use aesynx_boot::{BootInfo, MemoryAccountingError};

pub const BOOT_BANNER: &str = "Aesynx: booting";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootSummary {
    pub arch_label: &'static str,
    pub platform_label: &'static str,
    pub memory_regions: usize,
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
    pub rsdp_present: bool,
    pub framebuffer_present: bool,
    pub hhdm_present: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootSummaryError {
    MemoryAccounting(MemoryAccountingError),
}

pub fn boot_summary(info: &BootInfo<'_>) -> Result<BootSummary, BootSummaryError> {
    let memory = info
        .memory_map
        .summary()
        .map_err(BootSummaryError::MemoryAccounting)?;
    Ok(BootSummary {
        arch_label: arch_label(info),
        platform_label: platform_label(info),
        memory_regions: memory.region_count,
        total_bytes: memory.total_bytes,
        total_frames: memory.total_frames,
        usable_regions: memory.usable_regions,
        usable_bytes: memory.usable_bytes,
        usable_frames: memory.usable_frames,
        reserved_regions: memory.reserved_regions,
        reserved_bytes: memory.reserved_bytes,
        reserved_frames: memory.reserved_frames,
        kernel_bytes: memory.kernel_bytes,
        bootloader_bytes: memory.bootloader_bytes,
        framebuffer_bytes: memory.framebuffer_bytes,
        acpi_bytes: memory.acpi_bytes,
        bad_bytes: memory.bad_bytes,
        rsdp_present: info.rsdp_present(),
        framebuffer_present: info.framebuffer_present(),
        hhdm_present: info.hhdm_present(),
    })
}

fn arch_label(info: &BootInfo<'_>) -> &'static str {
    match info.arch {
        aesynx_boot::ArchKind::X86_64 => "arch=x86_64",
        aesynx_boot::ArchKind::Aarch64 => "arch=aarch64",
        aesynx_boot::ArchKind::Unknown => "arch=unknown",
    }
}

fn platform_label(info: &BootInfo<'_>) -> &'static str {
    match info.platform {
        aesynx_boot::PlatformKind::Qemu => "platform=qemu",
        aesynx_boot::PlatformKind::Uefi => "platform=uefi",
        aesynx_boot::PlatformKind::Unknown => "platform=unknown",
    }
}

#[cfg(test)]
mod tests {
    use aesynx_abi::{PhysAddr, VirtAddr};
    use aesynx_boot::{
        ArchKind, BootInfo, BootMetadata, HhdmInfo, KernelImageInfo, MemoryRegion,
        MemoryRegionKind, PlatformKind,
    };

    use super::boot_summary;

    #[test]
    fn boot_summary_uses_normalized_bootinfo() -> Result<(), aesynx_boot::BootInfoError> {
        let regions = [
            MemoryRegion::new(PhysAddr::new(0x1000), 0x1000, MemoryRegionKind::Usable),
            MemoryRegion::new(PhysAddr::new(0x3000), 0x2000, MemoryRegionKind::Usable),
            MemoryRegion::new(PhysAddr::new(0x8000), 0x1000, MemoryRegionKind::Bootloader),
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
            hhdm: Some(HhdmInfo::new(VirtAddr::new(0xffff800000000000))),
        })?;

        let summary = boot_summary(&info)
            .map_err(|_error| aesynx_boot::BootInfoError::InvalidMemoryRegion)?;

        assert_eq!(summary.arch_label, "arch=x86_64");
        assert_eq!(summary.platform_label, "platform=qemu");
        assert_eq!(summary.memory_regions, 3);
        assert_eq!(summary.total_bytes, 0x4000);
        assert_eq!(summary.total_frames, 4);
        assert_eq!(summary.usable_regions, 2);
        assert_eq!(summary.usable_bytes, 0x3000);
        assert_eq!(summary.usable_frames, 3);
        assert_eq!(summary.reserved_regions, 1);
        assert_eq!(summary.reserved_bytes, 0x1000);
        assert_eq!(summary.reserved_frames, 1);
        assert_eq!(summary.bootloader_bytes, 0x1000);
        assert!(summary.rsdp_present);
        assert!(summary.hhdm_present);
        assert!(!summary.framebuffer_present);
        Ok(())
    }
}
