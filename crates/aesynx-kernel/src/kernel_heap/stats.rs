#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelHeapStatus {
    pub heap_bytes: usize,
    pub allocated_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub slab_classes: usize,
    pub slab_allocations: usize,
    pub page_allocations: usize,
    pub frees: usize,
    pub double_free_detected: bool,
    pub invalid_free_detected: bool,
    pub accounting_overflow_detected: bool,
    pub corrupt_free_list_detected: bool,
    pub box_ok: bool,
    pub vec_ok: bool,
    pub btree_ok: bool,
    pub slab_reuse_ok: bool,
    pub page_run_ok: bool,
    pub stress_ok: bool,
    pub oom_rejected: bool,
}

pub struct KernelHeapStats {
    pub heap_bytes: usize,
    pub allocated_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub slab_allocations: usize,
    pub page_allocations: usize,
    pub frees: usize,
    pub double_free_detected: bool,
    pub invalid_free_detected: bool,
    pub accounting_overflow_detected: bool,
    pub corrupt_free_list_detected: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelHeapError {
    AlreadyInitialized,
    CorruptFreeList,
    DoubleFree,
    InvalidFree,
    InvalidLayout,
    NotInitialized,
    OutOfMemory,
    ReentrantLock,
    SmokeFailed,
}
