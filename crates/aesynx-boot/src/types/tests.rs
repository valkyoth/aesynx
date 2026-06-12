use alloc::format;

use aesynx_abi::{PhysAddr, VirtAddr};

use super::{
    ArchKind, BootInfo, BootInfoParts, CpuTopology, FRAME_SIZE, HhdmInfo, KernelImageInfo,
    MemoryMap, MemoryRegion, MemoryRegionKind, PlatformKind,
};

#[test]
fn kernel_image_translates_virtual_address_to_physical_offset() {
    let image = KernelImageInfo::new(
        VirtAddr::new(0xffff_ffff_8000_0000),
        VirtAddr::new(0xffff_ffff_8001_0000),
        PhysAddr::new(0x0040_0000),
    );

    assert_eq!(
        image.phys_for_virt(VirtAddr::new(0xffff_ffff_8000_3000)),
        Some(PhysAddr::new(0x0040_3000))
    );
    assert_eq!(
        image.phys_for_virt(VirtAddr::new(0xffff_ffff_8001_0000)),
        None
    );
}

#[test]
fn kernel_image_debug_remains_redacted() {
    let image = KernelImageInfo::new(
        VirtAddr::new(0xffff_ffff_8000_0000),
        VirtAddr::new(0xffff_ffff_8001_0000),
        PhysAddr::new(0x0040_0000),
    );
    let debug = format!("{image:?}");

    assert_eq!(debug, "KernelImageInfo(redacted)");
    assert!(!debug.contains("8000"));
    assert!(!debug.contains("0040"));
}

#[test]
fn bootinfo_exposes_hhdm_offset_without_debug_leakage() {
    let region = MemoryRegion::new(PhysAddr::new(0x1000), FRAME_SIZE, MemoryRegionKind::Usable);
    let memory = [region];
    let info = BootInfo::new(BootInfoParts {
        arch: ArchKind::X86_64,
        platform: PlatformKind::Qemu,
        memory_map: MemoryMap::new(&memory),
        framebuffer: None,
        rsdp: None,
        device_tree: None,
        cpu_topology: CpuTopology::new(&[]),
        kernel_image: KernelImageInfo::new(
            VirtAddr::new(0xffff_ffff_8000_0000),
            VirtAddr::new(0xffff_ffff_8001_0000),
            PhysAddr::new(0x0040_0000),
        ),
        hhdm: Some(HhdmInfo::new(VirtAddr::new(0xffff_8000_0000_0000))),
    });
    let debug = format!("{info:?}");

    assert_eq!(
        info.hhdm_offset(),
        Some(VirtAddr::new(0xffff_8000_0000_0000))
    );
    assert!(debug.contains("hhdm_present: true"));
    assert!(!debug.contains("ffff800000000000"));
}
