use core::alloc::{GlobalAlloc, Layout};

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
use aesynx_arch::ArchCpu;

use super::allocator::KernelHeapAllocator;
use super::stats::KernelHeapError;

// SAFETY: `KernelHeapAllocator` serializes metadata mutation with a private
// IRQ-masking spin lock, hands out nonoverlapping blocks from a private
// page-aligned static heap, and reconstructs the allocation class from the
// `Layout` supplied by `GlobalAlloc::dealloc`.
unsafe impl GlobalAlloc for KernelHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocate_checked(layout)
            .unwrap_or(core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Err(
            error @ (KernelHeapError::CorruptFreeList
            | KernelHeapError::DoubleFree
            | KernelHeapError::InvalidFree),
        ) = self.deallocate_checked(ptr, layout)
        {
            fail_stop_heap_corruption(error);
        }
    }
}

fn fail_stop_heap_corruption(error: KernelHeapError) -> ! {
    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    {
        aesynx_arch_x86_64::serial_println!("heap corruption detected error={:?}", error);
        aesynx_arch_x86_64::X86_64::halt_forever()
    }

    #[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
    {
        let _ = error;
        loop {
            core::hint::spin_loop();
        }
    }
}
