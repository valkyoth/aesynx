use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{
    ArchKind, BootInfo, BootInfoError, BootMetadata, KernelImageInfo, MemoryRegion,
    MemoryRegionKind, MemorySummary, PlatformKind,
};

fn qemu_metadata<'a>(memory_regions: &'a [MemoryRegion]) -> BootMetadata<'a> {
    BootMetadata {
        arch: ArchKind::X86_64,
        platform: PlatformKind::Qemu,
        memory_regions,
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
    }
}

#[test]
fn bootinfo_normalizes_synthetic_memory_map() -> Result<(), BootInfoError> {
    let regions = [
        MemoryRegion::new(PhysAddr::new(0x1000), 0x9000, MemoryRegionKind::Usable),
        MemoryRegion::new(PhysAddr::new(0x10000), 0x2000, MemoryRegionKind::Bootloader),
        MemoryRegion::new(PhysAddr::new(0x200000), 0x2000, MemoryRegionKind::Kernel),
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
        Ok(MemorySummary {
            region_count: 3,
            total_bytes: 0xd000,
            total_frames: 13,
            usable_regions: 1,
            usable_bytes: 0x9000,
            usable_frames: 9,
            reserved_regions: 2,
            reserved_bytes: 0x4000,
            reserved_frames: 4,
            kernel_bytes: 0x2000,
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
fn bootinfo_accepts_adjacent_memory_regions() -> Result<(), BootInfoError> {
    let regions = [
        MemoryRegion::new(PhysAddr::new(0x1000), 0x1ff000, MemoryRegionKind::Usable),
        MemoryRegion::new(PhysAddr::new(0x200000), 0x2000, MemoryRegionKind::Kernel),
    ];
    BootInfo::normalize(qemu_metadata(&regions))?;

    Ok(())
}

#[test]
fn bootinfo_accepts_kernel_image_split_across_reserved_regions() -> Result<(), BootInfoError> {
    let regions = [
        MemoryRegion::new(PhysAddr::new(0x1000), 0x1ff000, MemoryRegionKind::Usable),
        MemoryRegion::new(PhysAddr::new(0x200000), 0x1000, MemoryRegionKind::Reserved),
        MemoryRegion::new(PhysAddr::new(0x201000), 0x1000, MemoryRegionKind::Kernel),
    ];
    BootInfo::normalize(qemu_metadata(&regions))?;

    Ok(())
}

#[test]
fn bootinfo_rejects_kernel_image_inside_usable_memory() {
    let regions = [MemoryRegion::new(
        PhysAddr::new(0x200000),
        0x2000,
        MemoryRegionKind::Usable,
    )];
    let result = BootInfo::normalize(qemu_metadata(&regions));

    assert_eq!(result, Err(BootInfoError::KernelImageMemoryMapMismatch));
}

#[test]
fn bootinfo_rejects_kernel_image_without_reserved_memory_coverage() {
    let regions = [
        MemoryRegion::new(PhysAddr::new(0x1000), 0x1ff000, MemoryRegionKind::Usable),
        MemoryRegion::new(PhysAddr::new(0x200000), 0x1000, MemoryRegionKind::Kernel),
    ];
    let result = BootInfo::normalize(qemu_metadata(&regions));

    assert_eq!(result, Err(BootInfoError::KernelImageMemoryMapMismatch));
}

#[test]
fn bootinfo_rejects_unaligned_kernel_image_end() {
    let regions = [MemoryRegion::new(
        PhysAddr::new(0x200000),
        0x2000,
        MemoryRegionKind::Kernel,
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
            VirtAddr::new(0xffffffff80000000),
            VirtAddr::new(0xffffffff80001001),
            PhysAddr::new(0x200000),
        ),
        hhdm: None,
    });

    assert_eq!(result, Err(BootInfoError::KernelImageEmpty));
}
