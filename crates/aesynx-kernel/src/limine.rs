use aesynx_abi::{PhysAddr, VirtAddr};
use aesynx_boot::{
    ArchKind, BootInfo, BootMetadata, FramebufferInfo, HhdmInfo, KernelImageInfo,
    MAX_EARLY_MEMORY_REGIONS, MemoryRegion, MemoryRegionKind, PlatformKind,
};

pub struct EarlyBootScratch {
    memory_regions: [MemoryRegion; MAX_EARLY_MEMORY_REGIONS],
}

impl EarlyBootScratch {
    pub const fn new() -> Self {
        Self {
            memory_regions: [MemoryRegion::EMPTY; MAX_EARLY_MEMORY_REGIONS],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimineError {
    UnsupportedBaseRevision,
    MissingMemoryMap,
    MissingExecutableAddress,
    TooManyMemoryRegions,
    NullMemoryRegion,
    InvalidFramebuffer,
    BootInfoInvalid,
}

pub fn normalize<'a>(scratch: &'a mut EarlyBootScratch) -> Result<BootInfo<'a>, LimineError> {
    if !base_revision_supported() {
        return Err(LimineError::UnsupportedBaseRevision);
    }

    let memory_region_count = read_memory_regions(&mut scratch.memory_regions)?;
    let kernel_image = read_kernel_image()?;

    BootInfo::normalize(BootMetadata {
        arch: ArchKind::X86_64,
        platform: PlatformKind::Qemu,
        memory_regions: &scratch.memory_regions[..memory_region_count],
        framebuffer: read_framebuffer()?,
        rsdp: read_rsdp(),
        device_tree: None,
        cpu_topology: &[],
        kernel_image,
        hhdm: read_hhdm(),
    })
    .map_err(|_error| LimineError::BootInfoInvalid)
}

fn base_revision_supported() -> bool {
    // SAFETY: `BASE_REVISION` is a Limine-owned static tag. The bootloader writes
    // the third field before entering `_start`; reading it by raw pointer avoids
    // creating a reference to mutable bootloader-owned storage.
    let supported = unsafe {
        core::ptr::addr_of_mut!(BASE_REVISION)
            .cast::<u64>()
            .add(2)
            .read_volatile()
    };
    supported == 0
}

fn read_memory_regions(
    output: &mut [MemoryRegion; MAX_EARLY_MEMORY_REGIONS],
) -> Result<usize, LimineError> {
    // SAFETY: Limine fills `MEMMAP_REQUEST.response` before `_start`. The request
    // static lives in the retained Limine request section and is not mutated by
    // Aesynx after handoff.
    let response =
        unsafe { request_response::<LimineMemmapResponse>(core::ptr::addr_of!(MEMMAP_REQUEST)) }
            .ok_or(LimineError::MissingMemoryMap)?;

    let count = usize::try_from(response.entry_count)
        .map_err(|_error| LimineError::TooManyMemoryRegions)?;
    if count > output.len() {
        return Err(LimineError::TooManyMemoryRegions);
    }

    let entries = response.entries as *const *const LimineMemmapEntry;
    if entries.is_null() || !entries.is_aligned() {
        return Err(LimineError::MissingMemoryMap);
    }

    let mut index = 0usize;
    while index < count {
        // SAFETY: Limine reports `entry_count` pointers in the `entries` array.
        // Each pointer is expected to reference a valid memmap entry in
        // bootloader-reclaimable memory for the duration of early boot.
        let entry_ptr = unsafe { entries.add(index).read() };
        // SAFETY: Limine supplied this pointer through the bounded memmap
        // entries array. `limine_ref` performs null and alignment checks before
        // creating a reference.
        let entry = if let Some(entry) = unsafe { limine_ref(entry_ptr) } {
            entry
        } else {
            return Err(LimineError::NullMemoryRegion);
        };

        output[index] = MemoryRegion::new(
            PhysAddr::new(entry.base),
            entry.length,
            map_memory_region_kind(entry.kind),
        );
        index += 1;
    }

    Ok(count)
}

fn read_kernel_image() -> Result<KernelImageInfo, LimineError> {
    // SAFETY: Limine fills `EXECUTABLE_ADDRESS_REQUEST.response` before `_start`.
    // The response contains plain address values and is read-only after handoff.
    let response = unsafe {
        request_response::<LimineExecutableAddressResponse>(core::ptr::addr_of!(
            EXECUTABLE_ADDRESS_REQUEST
        ))
    }
    .ok_or(LimineError::MissingExecutableAddress)?;

    Ok(KernelImageInfo::new(
        VirtAddr::new(response.virtual_base),
        VirtAddr::new(kernel_virt_end()),
        PhysAddr::new(response.physical_base),
    ))
}

fn read_hhdm() -> Option<HhdmInfo> {
    // SAFETY: Limine fills `HHDM_REQUEST.response` before `_start` when the
    // feature is available. A missing response is represented as null.
    let response =
        unsafe { request_response::<LimineHhdmResponse>(core::ptr::addr_of!(HHDM_REQUEST)) }?;
    Some(HhdmInfo::new(VirtAddr::new(response.offset)))
}

fn read_framebuffer() -> Result<Option<FramebufferInfo>, LimineError> {
    // SAFETY: Limine fills `FRAMEBUFFER_REQUEST.response` before `_start` when
    // the feature is available. A missing response is represented as null.
    let Some(response) = (unsafe {
        request_response::<LimineFramebufferResponse>(core::ptr::addr_of!(FRAMEBUFFER_REQUEST))
    }) else {
        return Ok(None);
    };

    if response.framebuffer_count == 0 {
        return Ok(None);
    }

    let framebuffers = response.framebuffers as *const *const LimineFramebuffer;
    if framebuffers.is_null() || !framebuffers.is_aligned() {
        return Err(LimineError::InvalidFramebuffer);
    }

    // SAFETY: Limine reports at least one framebuffer pointer. We only consume
    // the first one during early boot and validate lossy integer conversions
    // below.
    let framebuffer_ptr = unsafe { framebuffers.read() };
    // SAFETY: Limine supplied this pointer through the framebuffer array.
    // `limine_ref` performs null and alignment checks before creating a
    // reference.
    let framebuffer = if let Some(framebuffer) = unsafe { limine_ref(framebuffer_ptr) } {
        framebuffer
    } else {
        return Err(LimineError::InvalidFramebuffer);
    };

    let base = framebuffer.address as u64;
    let width =
        u32::try_from(framebuffer.width).map_err(|_error| LimineError::InvalidFramebuffer)?;
    let height =
        u32::try_from(framebuffer.height).map_err(|_error| LimineError::InvalidFramebuffer)?;
    let stride =
        u32::try_from(framebuffer.pitch).map_err(|_error| LimineError::InvalidFramebuffer)?;

    Ok(Some(FramebufferInfo::new(
        VirtAddr::new(base),
        width,
        height,
        stride,
    )))
}

fn read_rsdp() -> Option<VirtAddr> {
    // SAFETY: Limine fills `RSDP_REQUEST.response` before `_start` when the
    // feature is available. A missing response is represented as null.
    let response =
        unsafe { request_response::<LimineRsdpResponse>(core::ptr::addr_of!(RSDP_REQUEST)) }?;
    Some(VirtAddr::new(response.address as u64))
}

fn map_memory_region_kind(kind: u64) -> MemoryRegionKind {
    match kind {
        LIMINE_MEMMAP_USABLE => MemoryRegionKind::Usable,
        LIMINE_MEMMAP_ACPI_RECLAIMABLE | LIMINE_MEMMAP_ACPI_NVS => MemoryRegionKind::Acpi,
        LIMINE_MEMMAP_BAD_MEMORY => MemoryRegionKind::Bad,
        LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE => MemoryRegionKind::Bootloader,
        LIMINE_MEMMAP_EXECUTABLE_AND_MODULES => MemoryRegionKind::Kernel,
        LIMINE_MEMMAP_FRAMEBUFFER => MemoryRegionKind::Framebuffer,
        LIMINE_MEMMAP_RESERVED | LIMINE_MEMMAP_RESERVED_MAPPED => MemoryRegionKind::Reserved,
        _unknown => MemoryRegionKind::Reserved,
    }
}

unsafe fn request_response<T>(request: *const LimineRequest) -> Option<&'static T> {
    // SAFETY: The caller provides the address of a live Limine request static.
    // Reading the response word is volatile because the bootloader wrote it
    // outside Rust's aliasing model before transferring control to `_start`.
    let response = unsafe { core::ptr::addr_of!((*request).response).read_volatile() };
    let response = response as *const T;
    // SAFETY: A non-null, aligned response pointer is owned by Limine and
    // remains valid in bootloader-reclaimable memory during this early boot
    // normalization phase.
    unsafe { limine_ref(response) }
}

unsafe fn limine_ref<T>(ptr: *const T) -> Option<&'static T> {
    if ptr.is_null() || !ptr.is_aligned() {
        return None;
    }

    // SAFETY: The caller established that this pointer came from Limine response
    // memory and remains live during early boot. This helper additionally guards
    // against null and misaligned pointers before creating a reference.
    Some(unsafe { &*ptr })
}

fn kernel_virt_end() -> u64 {
    unsafe extern "C" {
        static __kernel_end: u8;
    }

    // SAFETY: `__kernel_end` is provided by the kernel linker script. Taking its
    // address does not read memory or create a mutable alias.
    core::ptr::addr_of!(__kernel_end) as u64
}

#[repr(C)]
struct LimineRequest {
    id: [u64; 4],
    revision: u64,
    response: u64,
}

#[repr(C)]
struct LimineMemmapResponse {
    revision: u64,
    entry_count: u64,
    entries: u64,
}

#[repr(C)]
struct LimineMemmapEntry {
    base: u64,
    length: u64,
    kind: u64,
}

#[repr(C)]
struct LimineExecutableAddressResponse {
    revision: u64,
    physical_base: u64,
    virtual_base: u64,
}

#[repr(C)]
struct LimineHhdmResponse {
    revision: u64,
    offset: u64,
}

#[repr(C)]
struct LimineFramebufferResponse {
    revision: u64,
    framebuffer_count: u64,
    framebuffers: u64,
}

#[repr(C)]
struct LimineFramebuffer {
    address: *mut u8,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
    memory_model: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
    unused: [u8; 7],
    edid_size: u64,
    edid: u64,
    mode_count: u64,
    modes: u64,
}

const _: () = assert!(
    core::mem::size_of::<LimineFramebuffer>() == 80,
    "LimineFramebuffer size does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineFramebuffer, edid_size) == 48,
    "LimineFramebuffer.edid_size offset does not match Limine protocol ABI"
);

#[repr(C)]
struct LimineRsdpResponse {
    revision: u64,
    address: *const u8,
}

const LIMINE_COMMON_MAGIC0: u64 = 0xc7b1dd30df4c8b88;
const LIMINE_COMMON_MAGIC1: u64 = 0x0a82e883a194f07b;
const LIMINE_MEMMAP_USABLE: u64 = 0;
const LIMINE_MEMMAP_RESERVED: u64 = 1;
const LIMINE_MEMMAP_ACPI_RECLAIMABLE: u64 = 2;
const LIMINE_MEMMAP_ACPI_NVS: u64 = 3;
const LIMINE_MEMMAP_BAD_MEMORY: u64 = 4;
const LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE: u64 = 5;
const LIMINE_MEMMAP_EXECUTABLE_AND_MODULES: u64 = 6;
const LIMINE_MEMMAP_FRAMEBUFFER: u64 = 7;
const LIMINE_MEMMAP_RESERVED_MAPPED: u64 = 8;

#[used]
#[unsafe(link_section = ".limine_requests_start")]
static REQUESTS_START: [u64; 4] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
];

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut BASE_REVISION: [u64; 3] = [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 6];

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut MEMMAP_REQUEST: LimineRequest = LimineRequest {
    id: [
        LIMINE_COMMON_MAGIC0,
        LIMINE_COMMON_MAGIC1,
        0x67cf3d9d378a806f,
        0xe304acdfc50c3c62,
    ],
    revision: 0,
    response: 0,
};

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut EXECUTABLE_ADDRESS_REQUEST: LimineRequest = LimineRequest {
    id: [
        LIMINE_COMMON_MAGIC0,
        LIMINE_COMMON_MAGIC1,
        0x71ba76863cc55f63,
        0xb2644a48c516a487,
    ],
    revision: 0,
    response: 0,
};

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut HHDM_REQUEST: LimineRequest = LimineRequest {
    id: [
        LIMINE_COMMON_MAGIC0,
        LIMINE_COMMON_MAGIC1,
        0x48dcf1cb8ad2b852,
        0x63984e959a98244b,
    ],
    revision: 0,
    response: 0,
};

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut FRAMEBUFFER_REQUEST: LimineRequest = LimineRequest {
    id: [
        LIMINE_COMMON_MAGIC0,
        LIMINE_COMMON_MAGIC1,
        0x9d5827dcd881dd75,
        0xa3148604f6fab11b,
    ],
    revision: 0,
    response: 0,
};

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut RSDP_REQUEST: LimineRequest = LimineRequest {
    id: [
        LIMINE_COMMON_MAGIC0,
        LIMINE_COMMON_MAGIC1,
        0xc5e77b6b397e7b43,
        0x27637845accdcf3c,
    ],
    revision: 0,
    response: 0,
};

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static REQUESTS_END: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];
