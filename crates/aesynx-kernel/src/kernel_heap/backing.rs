use super::layout::AlignedKernelHeap;

pub(super) fn kernel_heap_start() -> usize {
    // SAFETY: Taking the raw address of the private static heap does not read
    // or write the heap and does not construct a Rust reference. The address is
    // used only as a numeric allocator bound during one-shot initialization.
    let heap = unsafe { core::ptr::addr_of_mut!(KERNEL_HEAP.bytes) as *mut u8 };
    core::hint::black_box(heap as usize)
}

#[unsafe(link_section = ".aesynx_kernel_heap")]
#[used]
static mut KERNEL_HEAP: AlignedKernelHeap = AlignedKernelHeap::ZERO;
