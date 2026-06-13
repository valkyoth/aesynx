#![cfg_attr(
    any(
        feature = "panic-smoke",
        feature = "exception-smoke",
        feature = "timer-smoke"
    ),
    allow(dead_code)
)]

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

pub const EARLY_HEAP_BYTES: usize = 64 * 1024;
const HEAP_UNINITIALIZED: usize = 0;
const HEAP_INITIALIZING: usize = 1;
const HEAP_INITIALIZED: usize = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EarlyHeapStatus {
    pub heap_bytes: usize,
    pub allocated_bytes: usize,
    pub box_ok: bool,
    pub vec_ok: bool,
    pub btree_ok: bool,
    pub oom_rejected: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EarlyHeapError {
    AlreadyInitialized,
    InvalidLayout,
    NotInitialized,
    SmokeFailed,
}

#[repr(C, align(4096))]
struct AlignedEarlyHeap {
    bytes: [u8; EARLY_HEAP_BYTES],
}

impl AlignedEarlyHeap {
    const ZERO: Self = Self {
        bytes: [0; EARLY_HEAP_BYTES],
    };
}

pub struct EarlyBumpAllocator {
    start: AtomicUsize,
    end: AtomicUsize,
    next: AtomicUsize,
    state: AtomicUsize,
}

impl EarlyBumpAllocator {
    pub const fn new() -> Self {
        Self {
            start: AtomicUsize::new(0),
            end: AtomicUsize::new(0),
            next: AtomicUsize::new(0),
            state: AtomicUsize::new(HEAP_UNINITIALIZED),
        }
    }

    pub fn init(&self) -> Result<(), EarlyHeapError> {
        if self
            .state
            .compare_exchange(
                HEAP_UNINITIALIZED,
                HEAP_INITIALIZING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_err()
        {
            return Err(EarlyHeapError::AlreadyInitialized);
        }

        let start = early_heap_start();
        let Some(end) = start.checked_add(EARLY_HEAP_BYTES) else {
            self.state.store(HEAP_UNINITIALIZED, Ordering::Release);
            return Err(EarlyHeapError::InvalidLayout);
        };
        self.start.store(start, Ordering::Release);
        self.end.store(end, Ordering::Release);
        self.next.store(start, Ordering::Release);
        self.state.store(HEAP_INITIALIZED, Ordering::Release);
        Ok(())
    }

    pub fn allocated_bytes(&self) -> Result<usize, EarlyHeapError> {
        if self.state.load(Ordering::Acquire) != HEAP_INITIALIZED {
            return Err(EarlyHeapError::NotInitialized);
        }
        let start = self.start.load(Ordering::Acquire);
        let next = self.next.load(Ordering::Acquire);
        next.checked_sub(start).ok_or(EarlyHeapError::InvalidLayout)
    }

    fn allocate(&self, layout: Layout) -> *mut u8 {
        if self.state.load(Ordering::Acquire) != HEAP_INITIALIZED {
            return core::ptr::null_mut();
        }

        let size = layout.size();
        let align = layout.align();
        let end = self.end.load(Ordering::Acquire);
        let mut observed = self.next.load(Ordering::Acquire);

        loop {
            let Some(aligned) = align_up(observed, align) else {
                return core::ptr::null_mut();
            };
            let Some(next) = aligned.checked_add(size) else {
                return core::ptr::null_mut();
            };
            if next > end {
                return core::ptr::null_mut();
            }

            match self
                .next
                .compare_exchange(observed, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return aligned as *mut u8,
                Err(actual) => observed = actual,
            }
        }
    }
}

// SAFETY: `EarlyBumpAllocator` hands out monotonically increasing, nonoverlapping
// ranges from a single static, page-aligned heap region after one-shot
// initialization. Failed allocations return null and `dealloc` is intentionally
// a no-op for the v0.17 bump-only heap.
unsafe impl GlobalAlloc for EarlyBumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocate(layout)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

pub fn smoke(allocator: &EarlyBumpAllocator) -> Result<EarlyHeapStatus, EarlyHeapError> {
    allocator.init()?;

    let boxed = Box::new(0x0a17_0001_u64);
    let box_ok = *boxed == 0x0a17_0001_u64;

    let mut values = Vec::new();
    values
        .try_reserve_exact(4)
        .map_err(|_error| EarlyHeapError::SmokeFailed)?;
    values.push(3_u64);
    values.push(5_u64);
    values.push(8_u64);
    values.push(13_u64);
    let vec_ok = values.as_slice() == [3, 5, 8, 13];

    let mut map = BTreeMap::new();
    map.insert(2_u8, 3_u8);
    map.insert(5_u8, 8_u8);
    let btree_ok = map.get(&5).copied() == Some(8) && map.len() == 2;

    let mut oversized = Vec::<u8>::new();
    let oom_rejected = oversized.try_reserve_exact(EARLY_HEAP_BYTES * 2).is_err();

    if !(box_ok && vec_ok && btree_ok && oom_rejected) {
        return Err(EarlyHeapError::SmokeFailed);
    }

    Ok(EarlyHeapStatus {
        heap_bytes: EARLY_HEAP_BYTES,
        allocated_bytes: allocator.allocated_bytes()?,
        box_ok,
        vec_ok,
        btree_ok,
        oom_rejected,
    })
}

fn align_up(value: usize, align: usize) -> Option<usize> {
    let mask = align.checked_sub(1)?;
    value.checked_add(mask).map(|aligned| aligned & !mask)
}

fn early_heap_start() -> usize {
    // SAFETY: Taking the raw address of the private static heap does not read
    // or write the heap and does not construct a Rust reference. The address is
    // used only as a numeric allocator bound during one-shot initialization.
    let heap = unsafe { core::ptr::addr_of_mut!(EARLY_HEAP.bytes) as *mut u8 };
    core::hint::black_box(heap as usize)
}

#[unsafe(link_section = ".aesynx_early_heap")]
#[used]
static mut EARLY_HEAP: AlignedEarlyHeap = AlignedEarlyHeap::ZERO;
