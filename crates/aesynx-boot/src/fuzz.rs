use aesynx_abi::{PhysAddr, VirtAddr};

use crate::{
    ArchKind, BootInfo, BootInfoError, BootMetadata, FramebufferInfo, HhdmInfo, KernelImageInfo,
    MemoryRegion, MemoryRegionKind, PlatformKind,
};

const MAX_FUZZ_REGIONS: usize = 8;
const DEFAULT_X86_KERNEL_VIRT: u64 = 0xffff_ffff_8000_0000;
const DEFAULT_AARCH64_KERNEL_VIRT: u64 = 0xffff_0000_8000_0000;
const DEFAULT_KERNEL_LEN: u64 = 0x2000;
const DEFAULT_KERNEL_PHYS: u64 = 0x0020_0000;
const DEFAULT_HIGH_VIRT: u64 = 0xffff_8000_0000_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FuzzOutcome {
    accepted: bool,
    error: Option<BootInfoError>,
}

fn run_bootinfo_normalize_fuzz_target(bytes: &[u8]) -> FuzzOutcome {
    let mut cursor = ByteCursor::new(bytes);
    let arch = decode_arch(cursor.read_u8().unwrap_or(0));
    let platform = decode_platform(cursor.read_u8().unwrap_or(0));
    let region_count = usize::from(cursor.read_u8().unwrap_or(0)) % (MAX_FUZZ_REGIONS + 1);
    let mut regions = [MemoryRegion::EMPTY; MAX_FUZZ_REGIONS];

    let mut index = 0usize;
    while index < region_count {
        let default_start = ((index as u64) + 1) * 0x10_000;
        let start = cursor.read_u64().unwrap_or(default_start);
        let len = cursor.read_u64().unwrap_or(0x1000);
        let kind = decode_region_kind(cursor.read_u8().unwrap_or(index as u8));
        regions[index] = MemoryRegion::new(PhysAddr::new(start), len, kind);
        index += 1;
    }

    let metadata_flags = cursor.read_u8().unwrap_or(0);
    let kernel_virt = cursor
        .read_u64()
        .unwrap_or(default_kernel_virt_for_arch(arch));
    let kernel_len = cursor.read_u64().unwrap_or(DEFAULT_KERNEL_LEN);
    let kernel_phys = cursor.read_u64().unwrap_or(DEFAULT_KERNEL_PHYS);
    let kernel_end = kernel_virt.checked_add(kernel_len).unwrap_or(kernel_virt);
    let framebuffer = if metadata_flags & 0b0000_0001 != 0 {
        Some(FramebufferInfo::new(
            VirtAddr::new(cursor.read_u64().unwrap_or(DEFAULT_HIGH_VIRT + 0xb800)),
            cursor.read_u32().unwrap_or(80),
            cursor.read_u32().unwrap_or(25),
            cursor.read_u32().unwrap_or(80),
        ))
    } else {
        None
    };
    let rsdp = optional_virt(metadata_flags & 0b0000_0010 != 0, &mut cursor, 0x7000);
    let device_tree = optional_virt(metadata_flags & 0b0000_0100 != 0, &mut cursor, 0x8000);
    let hhdm = if metadata_flags & 0b0000_1000 != 0 {
        Some(HhdmInfo::new(VirtAddr::new(
            cursor.read_u64().unwrap_or(DEFAULT_HIGH_VIRT),
        )))
    } else {
        None
    };

    let metadata = BootMetadata {
        arch,
        platform,
        memory_regions: &regions[..region_count],
        framebuffer,
        rsdp,
        device_tree,
        cpu_topology: &[],
        kernel_image: KernelImageInfo::new(
            VirtAddr::new(kernel_virt),
            VirtAddr::new(kernel_end),
            PhysAddr::new(kernel_phys),
        ),
        hhdm,
    };

    match BootInfo::normalize(metadata) {
        Ok(info) => {
            assert!(!info.memory_map.is_empty());
            assert!(info.memory_map.summary().is_ok());
            assert!(info.kernel_image.virt_end().get() > info.kernel_image.virt_start().get());
            assert_ne!(info.kernel_image.phys_start().get(), 0);
            FuzzOutcome {
                accepted: true,
                error: None,
            }
        }
        Err(error) => FuzzOutcome {
            accepted: false,
            error: Some(error),
        },
    }
}

fn optional_virt(present: bool, cursor: &mut ByteCursor<'_>, offset: u64) -> Option<VirtAddr> {
    if present {
        Some(VirtAddr::new(
            cursor.read_u64().unwrap_or(DEFAULT_HIGH_VIRT + offset),
        ))
    } else {
        None
    }
}

fn default_kernel_virt_for_arch(arch: ArchKind) -> u64 {
    match arch {
        ArchKind::Aarch64 => DEFAULT_AARCH64_KERNEL_VIRT,
        ArchKind::X86_64 | ArchKind::Unknown => DEFAULT_X86_KERNEL_VIRT,
    }
}

fn decode_arch(value: u8) -> ArchKind {
    match value % 3 {
        0 => ArchKind::X86_64,
        1 => ArchKind::Aarch64,
        _ => ArchKind::Unknown,
    }
}

fn decode_platform(value: u8) -> PlatformKind {
    match value % 3 {
        0 => PlatformKind::Qemu,
        1 => PlatformKind::Uefi,
        _ => PlatformKind::Unknown,
    }
}

fn decode_region_kind(value: u8) -> MemoryRegionKind {
    match value % 7 {
        0 => MemoryRegionKind::Usable,
        1 => MemoryRegionKind::Reserved,
        2 => MemoryRegionKind::Kernel,
        3 => MemoryRegionKind::Bootloader,
        4 => MemoryRegionKind::Framebuffer,
        5 => MemoryRegionKind::Acpi,
        _ => MemoryRegionKind::Bad,
    }
}

struct ByteCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ByteCursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_u8(&mut self) -> Option<u8> {
        let value = self.bytes.get(self.offset).copied();
        if value.is_some() {
            self.offset += 1;
        }
        value
    }

    fn read_u32(&mut self) -> Option<u32> {
        let bytes = self.read_array::<4>()?;
        Some(u32::from_le_bytes(bytes))
    }

    fn read_u64(&mut self) -> Option<u64> {
        let bytes = self.read_array::<8>()?;
        Some(u64::from_le_bytes(bytes))
    }

    fn read_array<const LEN: usize>(&mut self) -> Option<[u8; LEN]> {
        let end = self.offset.checked_add(LEN)?;
        let slice = self.bytes.get(self.offset..end)?;
        let mut output = [0u8; LEN];
        output.copy_from_slice(slice);
        self.offset = end;
        Some(output)
    }
}

fn push_u8(output: &mut [u8], len: &mut usize, value: u8) {
    if *len < output.len() {
        output[*len] = value;
        *len += 1;
    }
}

fn push_u64(output: &mut [u8], len: &mut usize, value: u64) {
    for byte in value.to_le_bytes() {
        push_u8(output, len, byte);
    }
}

fn encoded_region_case(
    regions: &[(u64, u64, MemoryRegionKind)],
    arch: ArchKind,
    kernel_virt: u64,
    kernel_len: u64,
    kernel_phys: u64,
    metadata_flags: u8,
) -> ([u8; 192], usize) {
    let mut output = [0u8; 192];
    let mut len = 0usize;
    push_u8(&mut output, &mut len, encode_arch(arch));
    push_u8(&mut output, &mut len, 0);
    push_u8(&mut output, &mut len, regions.len() as u8);
    for (start, region_len, kind) in regions {
        push_u64(&mut output, &mut len, *start);
        push_u64(&mut output, &mut len, *region_len);
        push_u8(&mut output, &mut len, encode_region_kind(*kind));
    }
    push_u8(&mut output, &mut len, metadata_flags);
    push_u64(&mut output, &mut len, kernel_virt);
    push_u64(&mut output, &mut len, kernel_len);
    push_u64(&mut output, &mut len, kernel_phys);
    (output, len)
}

fn encode_arch(arch: ArchKind) -> u8 {
    match arch {
        ArchKind::X86_64 => 0,
        ArchKind::Aarch64 => 1,
        ArchKind::Unknown => 2,
    }
}

fn encode_region_kind(kind: MemoryRegionKind) -> u8 {
    match kind {
        MemoryRegionKind::Usable => 0,
        MemoryRegionKind::Reserved => 1,
        MemoryRegionKind::Kernel => 2,
        MemoryRegionKind::Bootloader => 3,
        MemoryRegionKind::Framebuffer => 4,
        MemoryRegionKind::Acpi => 5,
        MemoryRegionKind::Bad => 6,
    }
}

#[test]
fn bootinfo_fuzz_target_runs_named_seed_corpus() {
    let seeds = [
        (
            encoded_region_case(
                &[
                    (0x1000, 0x9000, MemoryRegionKind::Usable),
                    (0x20_0000, 0x2000, MemoryRegionKind::Kernel),
                ],
                ArchKind::X86_64,
                DEFAULT_X86_KERNEL_VIRT,
                DEFAULT_KERNEL_LEN,
                DEFAULT_KERNEL_PHYS,
                0b0000_1011,
            ),
            true,
        ),
        (
            encoded_region_case(
                &[],
                ArchKind::X86_64,
                DEFAULT_X86_KERNEL_VIRT,
                DEFAULT_KERNEL_LEN,
                DEFAULT_KERNEL_PHYS,
                0,
            ),
            false,
        ),
        (
            encoded_region_case(
                &[
                    (0x1000, 0x4000, MemoryRegionKind::Usable),
                    (0x3000, 0x2000, MemoryRegionKind::Kernel),
                ],
                ArchKind::X86_64,
                DEFAULT_X86_KERNEL_VIRT,
                DEFAULT_KERNEL_LEN,
                DEFAULT_KERNEL_PHYS,
                0,
            ),
            false,
        ),
        (
            encoded_region_case(
                &[
                    (0x1000, 0x4000, MemoryRegionKind::Usable),
                    (0x5000, 0x2000, MemoryRegionKind::Bootloader),
                ],
                ArchKind::X86_64,
                DEFAULT_X86_KERNEL_VIRT,
                DEFAULT_KERNEL_LEN,
                DEFAULT_KERNEL_PHYS,
                0,
            ),
            true,
        ),
        (
            encoded_region_case(
                &[(u64::MAX - 1, 2, MemoryRegionKind::Usable)],
                ArchKind::X86_64,
                DEFAULT_X86_KERNEL_VIRT,
                DEFAULT_KERNEL_LEN,
                DEFAULT_KERNEL_PHYS,
                0,
            ),
            false,
        ),
        (
            encoded_region_case(
                &[(0x1000, 0x9000, MemoryRegionKind::Usable)],
                ArchKind::X86_64,
                0x400000,
                DEFAULT_KERNEL_LEN,
                DEFAULT_KERNEL_PHYS,
                0,
            ),
            false,
        ),
        (
            encoded_region_case(
                &[(0x1000, 0x9000, MemoryRegionKind::Usable)],
                ArchKind::Aarch64,
                DEFAULT_AARCH64_KERNEL_VIRT,
                DEFAULT_KERNEL_LEN,
                DEFAULT_KERNEL_PHYS,
                0b0000_1000,
            ),
            true,
        ),
        (
            encoded_region_case(
                &[(0x1000, 0x9000, MemoryRegionKind::Usable)],
                ArchKind::X86_64,
                DEFAULT_X86_KERNEL_VIRT,
                u64::MAX,
                DEFAULT_KERNEL_PHYS,
                0,
            ),
            false,
        ),
    ];

    let mut accepted = 0usize;
    let mut rejected = 0usize;
    for ((bytes, len), expected_accepted) in seeds {
        let outcome = run_bootinfo_normalize_fuzz_target(&bytes[..len]);
        assert_eq!(outcome.accepted, expected_accepted);
        if outcome.accepted {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }

    assert!(accepted >= 2);
    assert!(rejected >= 3);
}

#[test]
fn bootinfo_fuzz_target_runs_deterministic_mutation_sweep() {
    let mut seed = 0xace5_0161_u64;
    let mut accepted = 0usize;
    let mut rejected = 0usize;

    let mut case_index = 0usize;
    while case_index < 128 {
        let mut bytes = [0u8; 96];
        let mut index = 0usize;
        while index < bytes.len() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            bytes[index] = (seed >> 32) as u8;
            index += 1;
        }
        let outcome = run_bootinfo_normalize_fuzz_target(&bytes);
        assert!(outcome.accepted || outcome.error.is_some());
        if outcome.accepted {
            accepted += 1;
        } else {
            rejected += 1;
        }
        case_index += 1;
    }

    assert_eq!(accepted + rejected, 128);
    assert_ne!(rejected, 0);
}
