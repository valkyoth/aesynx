pub(super) const X86_64_KERNEL_VMA_MIN: u64 = 0xffff_8000_0000_0000;
pub(super) const LIMINE_REQUEST_REVISION: u64 = 0;

pub(super) const LIMINE_MEMMAP_USABLE: u64 = 0;
pub(super) const LIMINE_MEMMAP_RESERVED: u64 = 1;
pub(super) const LIMINE_MEMMAP_ACPI_RECLAIMABLE: u64 = 2;
pub(super) const LIMINE_MEMMAP_ACPI_NVS: u64 = 3;
pub(super) const LIMINE_MEMMAP_BAD_MEMORY: u64 = 4;
pub(super) const LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE: u64 = 5;
pub(super) const LIMINE_MEMMAP_EXECUTABLE_AND_MODULES: u64 = 6;
pub(super) const LIMINE_MEMMAP_FRAMEBUFFER: u64 = 7;
pub(super) const LIMINE_MEMMAP_RESERVED_MAPPED: u64 = 8;

const LIMINE_COMMON_MAGIC0: u64 = 0xc7b1dd30df4c8b88;
const LIMINE_COMMON_MAGIC1: u64 = 0x0a82e883a194f07b;

#[repr(C)]
pub(super) struct LimineRequest {
    _id: [u64; 4],
    _revision: u64,
    pub(super) response: u64,
}

const _: () = assert!(
    core::mem::size_of::<LimineRequest>() == 48,
    "LimineRequest size does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineRequest, _revision) == 32,
    "LimineRequest.revision offset does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineRequest, response) == 40,
    "LimineRequest.response offset does not match Limine protocol ABI"
);

#[repr(C)]
pub(super) struct LimineMemmapResponse {
    pub(super) revision: u64,
    pub(super) entry_count: u64,
    pub(super) entries: u64,
}

const _: () = assert!(
    core::mem::size_of::<LimineMemmapResponse>() == 24,
    "LimineMemmapResponse size does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineMemmapResponse, entry_count) == 8,
    "LimineMemmapResponse.entry_count offset does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineMemmapResponse, entries) == 16,
    "LimineMemmapResponse.entries offset does not match Limine protocol ABI"
);

#[repr(C)]
pub(super) struct LimineMemmapEntry {
    pub(super) base: u64,
    pub(super) length: u64,
    pub(super) kind: u64,
}

const _: () = assert!(
    core::mem::size_of::<LimineMemmapEntry>() == 24,
    "LimineMemmapEntry size does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineMemmapEntry, length) == 8,
    "LimineMemmapEntry.length offset does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineMemmapEntry, kind) == 16,
    "LimineMemmapEntry.kind offset does not match Limine protocol ABI"
);

#[repr(C)]
pub(super) struct LimineExecutableAddressResponse {
    pub(super) revision: u64,
    pub(super) physical_base: u64,
    pub(super) virtual_base: u64,
}

const _: () = assert!(
    core::mem::size_of::<LimineExecutableAddressResponse>() == 24,
    "LimineExecutableAddressResponse size does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineExecutableAddressResponse, physical_base) == 8,
    "LimineExecutableAddressResponse.physical_base offset does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineExecutableAddressResponse, virtual_base) == 16,
    "LimineExecutableAddressResponse.virtual_base offset does not match Limine protocol ABI"
);

#[repr(C)]
pub(super) struct LimineHhdmResponse {
    pub(super) revision: u64,
    pub(super) offset: u64,
}

const _: () = assert!(
    core::mem::size_of::<LimineHhdmResponse>() == 16,
    "LimineHhdmResponse size does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineHhdmResponse, offset) == 8,
    "LimineHhdmResponse.offset does not match Limine protocol ABI"
);

#[repr(C)]
pub(super) struct LimineFramebufferResponse {
    pub(super) revision: u64,
    pub(super) framebuffer_count: u64,
    pub(super) framebuffers: u64,
}

const _: () = assert!(
    core::mem::size_of::<LimineFramebufferResponse>() == 24,
    "LimineFramebufferResponse size does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineFramebufferResponse, framebuffer_count) == 8,
    "LimineFramebufferResponse.framebuffer_count offset does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineFramebufferResponse, framebuffers) == 16,
    "LimineFramebufferResponse.framebuffers offset does not match Limine protocol ABI"
);

#[repr(C)]
pub(super) struct LimineFramebuffer {
    pub(super) address: *mut u8,
    pub(super) width: u64,
    pub(super) height: u64,
    pub(super) pitch: u64,
    _bpp: u16,
    _memory_model: u8,
    _red_mask_size: u8,
    _red_mask_shift: u8,
    _green_mask_size: u8,
    _green_mask_shift: u8,
    _blue_mask_size: u8,
    _blue_mask_shift: u8,
    _unused: [u8; 7],
    _edid_size: u64,
    _edid: u64,
    _mode_count: u64,
    _modes: u64,
}

const _: () = assert!(
    core::mem::size_of::<LimineFramebuffer>() == 80,
    "LimineFramebuffer size does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineFramebuffer, width) == 8,
    "LimineFramebuffer.width offset does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineFramebuffer, height) == 16,
    "LimineFramebuffer.height offset does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineFramebuffer, pitch) == 24,
    "LimineFramebuffer.pitch offset does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineFramebuffer, _edid_size) == 48,
    "LimineFramebuffer.edid_size offset does not match Limine protocol ABI"
);

#[repr(C)]
pub(super) struct LimineRsdpResponse {
    pub(super) revision: u64,
    pub(super) address: *const u8,
}

const _: () = assert!(
    core::mem::size_of::<LimineRsdpResponse>() == 16,
    "LimineRsdpResponse size does not match Limine protocol ABI"
);
const _: () = assert!(
    core::mem::offset_of!(LimineRsdpResponse, address) == 8,
    "LimineRsdpResponse.address offset does not match Limine protocol ABI"
);

pub(super) fn base_revision_ptr() -> *mut [u64; 3] {
    core::ptr::addr_of_mut!(BASE_REVISION)
}

pub(super) fn memmap_request_ptr() -> *const LimineRequest {
    core::ptr::addr_of!(MEMMAP_REQUEST)
}

pub(super) fn executable_address_request_ptr() -> *const LimineRequest {
    core::ptr::addr_of!(EXECUTABLE_ADDRESS_REQUEST)
}

pub(super) fn hhdm_request_ptr() -> *const LimineRequest {
    core::ptr::addr_of!(HHDM_REQUEST)
}

pub(super) fn framebuffer_request_ptr() -> *const LimineRequest {
    core::ptr::addr_of!(FRAMEBUFFER_REQUEST)
}

pub(super) fn rsdp_request_ptr() -> *const LimineRequest {
    core::ptr::addr_of!(RSDP_REQUEST)
}

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
    _id: [
        LIMINE_COMMON_MAGIC0,
        LIMINE_COMMON_MAGIC1,
        0x67cf3d9d378a806f,
        0xe304acdfc50c3c62,
    ],
    _revision: LIMINE_REQUEST_REVISION,
    response: 0,
};

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut EXECUTABLE_ADDRESS_REQUEST: LimineRequest = LimineRequest {
    _id: [
        LIMINE_COMMON_MAGIC0,
        LIMINE_COMMON_MAGIC1,
        0x71ba76863cc55f63,
        0xb2644a48c516a487,
    ],
    _revision: LIMINE_REQUEST_REVISION,
    response: 0,
};

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut HHDM_REQUEST: LimineRequest = LimineRequest {
    _id: [
        LIMINE_COMMON_MAGIC0,
        LIMINE_COMMON_MAGIC1,
        0x48dcf1cb8ad2b852,
        0x63984e959a98244b,
    ],
    _revision: LIMINE_REQUEST_REVISION,
    response: 0,
};

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut FRAMEBUFFER_REQUEST: LimineRequest = LimineRequest {
    _id: [
        LIMINE_COMMON_MAGIC0,
        LIMINE_COMMON_MAGIC1,
        0x9d5827dcd881dd75,
        0xa3148604f6fab11b,
    ],
    _revision: LIMINE_REQUEST_REVISION,
    response: 0,
};

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut RSDP_REQUEST: LimineRequest = LimineRequest {
    _id: [
        LIMINE_COMMON_MAGIC0,
        LIMINE_COMMON_MAGIC1,
        0xc5e77b6b397e7b43,
        0x27637845accdcf3c,
    ],
    _revision: LIMINE_REQUEST_REVISION,
    response: 0,
};

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static REQUESTS_END: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];
