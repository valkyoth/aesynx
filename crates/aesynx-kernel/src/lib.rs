#![no_std]
#![deny(unsafe_code)]

use aesynx_boot::BootInfo;
use aesynx_log::{LogLevel, LogMessage, LogSink};

pub const BOOT_BANNER: &str = "Aesynx: booting";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootSummary {
    pub arch_label: &'static str,
    pub platform_label: &'static str,
    pub memory_regions: usize,
    pub usable_regions: usize,
    pub usable_bytes: u64,
    pub rsdp_present: bool,
    pub framebuffer_present: bool,
    pub hhdm_present: bool,
}

#[must_use]
pub fn boot_summary(info: &BootInfo<'_>) -> BootSummary {
    let memory = info.memory_map.summary();
    BootSummary {
        arch_label: arch_label(info),
        platform_label: platform_label(info),
        memory_regions: memory.region_count,
        usable_regions: memory.usable_regions,
        usable_bytes: memory.usable_bytes,
        rsdp_present: info.rsdp.is_some(),
        framebuffer_present: info.framebuffer.is_some(),
        hhdm_present: info.hhdm.is_some(),
    }
}

pub fn describe_boot(info: &BootInfo<'_>, log: &impl LogSink) {
    let summary = boot_summary(info);
    write_boot_log(log, BOOT_BANNER);
    write_boot_log(log, summary.arch_label);
    write_boot_log(log, summary.platform_label);
}

fn write_boot_log(log: &impl LogSink, message: &'static str) {
    let message = LogMessage::new(message).unwrap_or(LogMessage::REJECTED);
    log.write_str(LogLevel::Info, "kernel", message);
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
            hhdm: Some(HhdmInfo {
                offset: VirtAddr::new(0xffff800000000000),
            }),
        })?;

        let summary = boot_summary(&info);

        assert_eq!(summary.arch_label, "arch=x86_64");
        assert_eq!(summary.platform_label, "platform=qemu");
        assert_eq!(summary.memory_regions, 3);
        assert_eq!(summary.usable_regions, 2);
        assert_eq!(summary.usable_bytes, 0x3000);
        assert!(summary.rsdp_present);
        assert!(summary.hhdm_present);
        assert!(!summary.framebuffer_present);
        Ok(())
    }
}
