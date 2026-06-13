pub const KERNEL_HEAP_BYTES: usize = 256 * 1024;
pub const KERNEL_HEAP_PAGE_SIZE: usize = 4096;
pub const KERNEL_HEAP_PAGES: usize = KERNEL_HEAP_BYTES / KERNEL_HEAP_PAGE_SIZE;
pub const SLAB_CLASS_COUNT: usize = 8;
pub const SLAB_CLASSES: [usize; SLAB_CLASS_COUNT] = [16, 32, 64, 128, 256, 512, 1024, 2048];

pub const PAGE_FREE: usize = 0;
pub const PAGE_LARGE_HEAD: usize = 1;
pub const PAGE_LARGE_TAIL: usize = 2;
pub const PAGE_SLAB_BASE: usize = 16;

#[repr(C, align(4096))]
pub struct AlignedKernelHeap {
    pub bytes: [u8; KERNEL_HEAP_BYTES],
}

impl AlignedKernelHeap {
    pub const ZERO: Self = Self {
        bytes: [0; KERNEL_HEAP_BYTES],
    };
}

pub const fn page_count_for_len(len: usize) -> usize {
    len / KERNEL_HEAP_PAGE_SIZE
}

pub const fn class_for_layout(size: usize, align: usize) -> Option<usize> {
    let requested = if size == 0 { 1 } else { size };
    let mut index = 0usize;
    while index < SLAB_CLASS_COUNT {
        let class = SLAB_CLASSES[index];
        if requested <= class && align <= class {
            return Some(index);
        }
        index += 1;
    }
    None
}

pub const fn align_up(value: usize, align: usize) -> Option<usize> {
    let Some(mask) = align.checked_sub(1) else {
        return None;
    };
    let Some(aligned) = value.checked_add(mask) else {
        return None;
    };
    Some(aligned & !mask)
}

pub const fn pages_for_size(size: usize) -> Option<usize> {
    let requested = if size == 0 { 1 } else { size };
    let Some(aligned) = align_up(requested, KERNEL_HEAP_PAGE_SIZE) else {
        return None;
    };
    Some(aligned / KERNEL_HEAP_PAGE_SIZE)
}
