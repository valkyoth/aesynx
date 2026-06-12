use aesynx_abi::VirtAddr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelSectionLayout {
    pub text_start: VirtAddr,
    pub text_end: VirtAddr,
    pub rodata_start: VirtAddr,
    pub rodata_end: VirtAddr,
    pub data_start: VirtAddr,
    pub data_end: VirtAddr,
}

pub fn layout() -> KernelSectionLayout {
    unsafe extern "C" {
        static __kernel_text_start: u8;
        static __kernel_text_end: u8;
        static __kernel_rodata_start: u8;
        static __kernel_rodata_end: u8;
        static __kernel_data_start: u8;
        static __kernel_data_end: u8;
    }

    // SAFETY: These symbols are provided by the kernel linker script. Taking
    // their addresses does not read memory or create mutable aliases.
    let text_start = core::ptr::addr_of!(__kernel_text_start) as u64;
    let text_end = core::ptr::addr_of!(__kernel_text_end) as u64;
    let rodata_start = core::ptr::addr_of!(__kernel_rodata_start) as u64;
    let rodata_end = core::ptr::addr_of!(__kernel_rodata_end) as u64;
    let data_start = core::ptr::addr_of!(__kernel_data_start) as u64;
    let data_end = core::ptr::addr_of!(__kernel_data_end) as u64;

    KernelSectionLayout {
        text_start: VirtAddr::new(text_start),
        text_end: VirtAddr::new(text_end),
        rodata_start: VirtAddr::new(rodata_start),
        rodata_end: VirtAddr::new(rodata_end),
        data_start: VirtAddr::new(data_start),
        data_end: VirtAddr::new(data_end),
    }
}
