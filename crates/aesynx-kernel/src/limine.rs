use aesynx_abi::{PhysAddr, VirtAddr};
use aesynx_boot::{
    ArchKind, BootInfo, BootMetadata, FramebufferInfo, HhdmInfo, KernelImageInfo,
    MAX_EARLY_MEMORY_REGIONS, MemoryRegion, MemoryRegionKind, PlatformKind,
};
use core::sync::atomic::{AtomicBool, Ordering};

mod abi;
use self::abi::{
    LimineExecutableAddressResponse, LimineFramebuffer, LimineFramebufferResponse,
    LimineHhdmResponse, LimineMemmapEntry, LimineMemmapResponse, LimineRsdpResponse,
};

const _: () = assert!(
    usize::BITS == 64,
    "Limine pointer address validation assumes a 64-bit target"
);

pub struct EarlyBootScratch {
    memory_regions: [MemoryRegion; MAX_EARLY_MEMORY_REGIONS],
}

static BOOTINFO_NORMALIZED: AtomicBool = AtomicBool::new(false);

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
    UnsupportedExecutableAddressRevision,
    UnsupportedFramebufferRevision,
    UnsupportedHhdmRevision,
    UnsupportedMemoryMapRevision,
    UnsupportedRsdpRevision,
    MissingMemoryMap,
    MissingExecutableAddress,
    TooManyMemoryRegions,
    NullMemoryRegion,
    InvalidHhdm,
    InvalidFramebuffer,
    InvalidRsdp,
    AlreadyNormalized,
    BootInfoInvalid,
}

pub fn normalize<'a>(scratch: &'a mut EarlyBootScratch) -> Result<BootInfo<'a>, LimineError> {
    claim_bootinfo_normalization_once()?;

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
        rsdp: read_rsdp()?,
        device_tree: None,
        cpu_topology: &[],
        kernel_image,
        hhdm: read_hhdm()?,
    })
    .map_err(|_error| LimineError::BootInfoInvalid)
}

fn claim_bootinfo_normalization_once() -> Result<(), LimineError> {
    if BOOTINFO_NORMALIZED.swap(true, Ordering::AcqRel) {
        return Err(LimineError::AlreadyNormalized);
    }
    Ok(())
}

#[cfg(test)]
fn reset_bootinfo_normalization_for_test() {
    BOOTINFO_NORMALIZED.store(false, Ordering::Release);
}

fn base_revision_supported() -> bool {
    // SAFETY: `BASE_REVISION` is a Limine-owned static tag. The bootloader writes
    // the third field before entering `_start`; reading it by raw pointer avoids
    // creating a reference to mutable bootloader-owned storage.
    let supported = unsafe {
        abi::base_revision_ptr()
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
    let response = unsafe {
        request_response::<LimineMemmapResponse>(
            abi::memmap_request_ptr(),
            abi::X86_64_KERNEL_VMA_MIN,
        )
    }
    .ok_or(LimineError::MissingMemoryMap)?;

    if !limine_response_revision_compatible(response.revision, abi::LIMINE_REQUEST_REVISION) {
        return Err(LimineError::UnsupportedMemoryMapRevision);
    }

    const _: () = assert!(
        usize::BITS >= 64,
        "Limine entry_count requires 64-bit usize"
    );
    let count = response.entry_count as usize;
    if count > output.len() {
        return Err(LimineError::TooManyMemoryRegions);
    }

    let entries = response.entries as *const *const LimineMemmapEntry;
    if !valid_handoff_array_ptr(entries, abi::X86_64_KERNEL_VMA_MIN) {
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
        let entry =
            if let Some(entry) = unsafe { limine_ref(entry_ptr, abi::X86_64_KERNEL_VMA_MIN) } {
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
        request_response::<LimineExecutableAddressResponse>(
            abi::executable_address_request_ptr(),
            abi::X86_64_KERNEL_VMA_MIN,
        )
    }
    .ok_or(LimineError::MissingExecutableAddress)?;

    if !limine_response_revision_compatible(response.revision, abi::LIMINE_REQUEST_REVISION) {
        return Err(LimineError::UnsupportedExecutableAddressRevision);
    }

    Ok(KernelImageInfo::new(
        VirtAddr::new(response.virtual_base),
        VirtAddr::new(kernel_virt_end()),
        PhysAddr::new(response.physical_base),
    ))
}

fn read_hhdm() -> Result<Option<HhdmInfo>, LimineError> {
    // SAFETY: Limine fills `HHDM_REQUEST.response` before `_start` when the
    // feature is available. A missing response is represented as null.
    let Some(response) = (unsafe {
        request_response::<LimineHhdmResponse>(abi::hhdm_request_ptr(), abi::X86_64_KERNEL_VMA_MIN)
    }) else {
        return Ok(None);
    };

    if !limine_response_revision_compatible(response.revision, abi::LIMINE_REQUEST_REVISION) {
        return Err(LimineError::UnsupportedHhdmRevision);
    }

    if !valid_handoff_virt(response.offset, abi::X86_64_KERNEL_VMA_MIN) {
        return Err(LimineError::InvalidHhdm);
    }

    Ok(Some(HhdmInfo::new(VirtAddr::new(response.offset))))
}

fn read_framebuffer() -> Result<Option<FramebufferInfo>, LimineError> {
    // SAFETY: Limine fills `FRAMEBUFFER_REQUEST.response` before `_start` when
    // the feature is available. A missing response is represented as null.
    let Some(response) = (unsafe {
        request_response::<LimineFramebufferResponse>(
            abi::framebuffer_request_ptr(),
            abi::X86_64_KERNEL_VMA_MIN,
        )
    }) else {
        return Ok(None);
    };

    if !limine_response_revision_compatible(response.revision, abi::LIMINE_REQUEST_REVISION) {
        return Err(LimineError::UnsupportedFramebufferRevision);
    }

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
    let framebuffer = if let Some(framebuffer) =
        unsafe { limine_ref(framebuffer_ptr, abi::X86_64_KERNEL_VMA_MIN) }
    {
        framebuffer
    } else {
        return Err(LimineError::InvalidFramebuffer);
    };

    // Provenance is intentionally dropped here: the framebuffer is bootloader
    // MMIO memory, not a Rust allocation. Future framebuffer writes must
    // re-acquire a raw pointer at the MMIO access site with a local safety
    // contract for the mapping lifetime and cacheability.
    let base = framebuffer.address as usize as u64;
    if !valid_handoff_virt(base, abi::X86_64_KERNEL_VMA_MIN) {
        return Err(LimineError::InvalidFramebuffer);
    }
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

fn read_rsdp() -> Result<Option<VirtAddr>, LimineError> {
    // SAFETY: Limine fills `RSDP_REQUEST.response` before `_start` when the
    // feature is available. A missing response is represented as null.
    let Some(response) = (unsafe {
        request_response::<LimineRsdpResponse>(abi::rsdp_request_ptr(), abi::X86_64_KERNEL_VMA_MIN)
    }) else {
        return Ok(None);
    };

    if !limine_response_revision_compatible(response.revision, abi::LIMINE_REQUEST_REVISION) {
        return Err(LimineError::UnsupportedRsdpRevision);
    }

    if response.address.is_null() {
        return Ok(None);
    }

    // Provenance is intentionally dropped here: ACPI consumes a firmware table
    // physical/virtual address, not a Rust allocation. Future ACPI readers must
    // re-acquire a raw pointer at the read site with a documented firmware
    // table lifetime contract.
    let address = response.address as usize as u64;
    if !valid_handoff_virt(address, abi::X86_64_KERNEL_VMA_MIN) {
        return Err(LimineError::InvalidRsdp);
    }

    Ok(Some(VirtAddr::new(address)))
}

fn map_memory_region_kind(kind: u64) -> MemoryRegionKind {
    match kind {
        abi::LIMINE_MEMMAP_USABLE => MemoryRegionKind::Usable,
        abi::LIMINE_MEMMAP_ACPI_RECLAIMABLE | abi::LIMINE_MEMMAP_ACPI_NVS => MemoryRegionKind::Acpi,
        abi::LIMINE_MEMMAP_BAD_MEMORY => MemoryRegionKind::Bad,
        abi::LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE => MemoryRegionKind::Bootloader,
        abi::LIMINE_MEMMAP_EXECUTABLE_AND_MODULES => MemoryRegionKind::Kernel,
        abi::LIMINE_MEMMAP_FRAMEBUFFER => MemoryRegionKind::Framebuffer,
        abi::LIMINE_MEMMAP_RESERVED | abi::LIMINE_MEMMAP_RESERVED_MAPPED => {
            MemoryRegionKind::Reserved
        }
        _unknown => MemoryRegionKind::Reserved,
    }
}

const fn limine_response_revision_compatible(
    response_revision: u64,
    request_revision: u64,
) -> bool {
    response_revision >= request_revision
}

unsafe fn request_response<T>(
    request: *const abi::LimineRequest,
    min_kernel_vma: u64,
) -> Option<&'static T> {
    // SAFETY: The caller provides the address of a live Limine request static.
    // Reading the response word is volatile because the bootloader wrote it
    // outside Rust's aliasing model before transferring control to `_start`.
    let response = unsafe { core::ptr::addr_of!((*request).response).read_volatile() };
    let response = response as *const T;
    // SAFETY: A non-null, aligned response pointer is owned by Limine and
    // remains valid in bootloader-reclaimable memory during this early boot
    // normalization phase.
    unsafe { limine_ref(response, min_kernel_vma) }
}

unsafe fn limine_ref<T>(ptr: *const T, min_kernel_vma: u64) -> Option<&'static T> {
    if ptr.is_null() || !ptr.is_aligned() {
        return None;
    }

    if !valid_handoff_virt(ptr as usize as u64, min_kernel_vma) {
        return None;
    }

    // SAFETY: The caller established that this pointer came from Limine response
    // memory and remains live during early boot. This helper additionally guards
    // against null, misaligned, and lower-half pointers before creating a
    // reference.
    Some(unsafe { &*ptr })
}

fn valid_handoff_array_ptr<T>(ptr: *const T, min_kernel_vma: u64) -> bool {
    !ptr.is_null() && ptr.is_aligned() && valid_handoff_virt(ptr as usize as u64, min_kernel_vma)
}

const fn valid_handoff_virt(address: u64, min_kernel_vma: u64) -> bool {
    address >= min_kernel_vma && is_canonical_address(address)
}

const fn is_canonical_address(address: u64) -> bool {
    let sign_bit = (address >> 47) & 1;
    let upper = address >> 48;
    (sign_bit == 0 && upper == 0) || (sign_bit == 1 && upper == 0xffff)
}

fn kernel_virt_end() -> u64 {
    unsafe extern "C" {
        static __kernel_end: u8;
    }

    // SAFETY: `__kernel_end` is provided by the kernel linker script. Taking its
    // address does not read memory or create a mutable alias.
    core::ptr::addr_of!(__kernel_end) as u64
}

#[cfg(test)]
mod tests;
