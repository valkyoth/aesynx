use core::alloc::Layout;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[cfg(feature = "smp")]
compile_error!(
    "KERNEL_HEAP uses single-core static backing storage; move heap storage to \
     explicit interior mutability or per-core ownership before enabling smp"
);

use super::backing::kernel_heap_start;
use super::free_list::{
    FREE_LIST_EMPTY, decode_offset, decode_valid_offset, encode_offset, read_free_next,
    write_free_next, zero_heap_bytes,
};
use super::layout::{
    KERNEL_HEAP_BYTES, KERNEL_HEAP_PAGE_SIZE, KERNEL_HEAP_PAGES, PAGE_FREE, PAGE_LARGE_HEAD,
    PAGE_LARGE_TAIL, PAGE_SLAB_BASE, SLAB_CLASS_COUNT, SLAB_CLASSES, class_for_layout,
    page_count_for_len, pages_for_size,
};
use super::lock::HeapLockGuard;
use super::stats::{KernelHeapError, KernelHeapStats};

const HEAP_UNINITIALIZED: usize = 0;
const HEAP_INITIALIZING: usize = 1;
const HEAP_INITIALIZED: usize = 2;

pub struct KernelHeapAllocator {
    start: AtomicUsize,
    total_pages: AtomicUsize,
    state: AtomicUsize,
    locked: AtomicBool,
    free_heads: [AtomicUsize; SLAB_CLASS_COUNT],
    page_state: [AtomicUsize; KERNEL_HEAP_PAGES],
    run_pages: [AtomicUsize; KERNEL_HEAP_PAGES],
    page_live_blocks: [AtomicUsize; KERNEL_HEAP_PAGES],
    allocated_bytes: AtomicUsize,
    peak_allocated_bytes: AtomicUsize,
    slab_allocations: AtomicUsize,
    page_allocations: AtomicUsize,
    frees: AtomicUsize,
    double_free_detected: AtomicUsize,
    invalid_free_detected: AtomicUsize,
    corrupt_free_list_detected: AtomicUsize,
}

impl KernelHeapAllocator {
    pub const fn new() -> Self {
        Self {
            start: AtomicUsize::new(0),
            total_pages: AtomicUsize::new(0),
            state: AtomicUsize::new(HEAP_UNINITIALIZED),
            locked: AtomicBool::new(false),
            free_heads: [const { AtomicUsize::new(FREE_LIST_EMPTY) }; SLAB_CLASS_COUNT],
            page_state: [const { AtomicUsize::new(PAGE_FREE) }; KERNEL_HEAP_PAGES],
            run_pages: [const { AtomicUsize::new(0) }; KERNEL_HEAP_PAGES],
            page_live_blocks: [const { AtomicUsize::new(0) }; KERNEL_HEAP_PAGES],
            allocated_bytes: AtomicUsize::new(0),
            peak_allocated_bytes: AtomicUsize::new(0),
            slab_allocations: AtomicUsize::new(0),
            page_allocations: AtomicUsize::new(0),
            frees: AtomicUsize::new(0),
            double_free_detected: AtomicUsize::new(0),
            invalid_free_detected: AtomicUsize::new(0),
            corrupt_free_list_detected: AtomicUsize::new(0),
        }
    }

    pub fn init(&self) -> Result<(), KernelHeapError> {
        self.init_with_bounds(kernel_heap_start(), KERNEL_HEAP_BYTES)
    }

    pub(crate) fn init_with_bounds(&self, start: usize, len: usize) -> Result<(), KernelHeapError> {
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
            return Err(KernelHeapError::AlreadyInitialized);
        }

        if start & (KERNEL_HEAP_PAGE_SIZE - 1) != 0 {
            self.state.store(HEAP_UNINITIALIZED, Ordering::Release);
            return Err(KernelHeapError::InvalidLayout);
        }
        let total_pages = page_count_for_len(len);
        if total_pages == 0 || total_pages > KERNEL_HEAP_PAGES {
            self.state.store(HEAP_UNINITIALIZED, Ordering::Release);
            return Err(KernelHeapError::InvalidLayout);
        }
        if start
            .checked_add(total_pages * KERNEL_HEAP_PAGE_SIZE)
            .is_none()
        {
            self.state.store(HEAP_UNINITIALIZED, Ordering::Release);
            return Err(KernelHeapError::InvalidLayout);
        }

        self.start.store(start, Ordering::Release);
        self.total_pages.store(total_pages, Ordering::Release);
        self.reset_metadata();
        self.state.store(HEAP_INITIALIZED, Ordering::Release);
        Ok(())
    }

    #[cfg(test)]
    pub fn allocated_bytes(&self) -> Result<usize, KernelHeapError> {
        if self.state.load(Ordering::Acquire) != HEAP_INITIALIZED {
            return Err(KernelHeapError::NotInitialized);
        }
        Ok(self.allocated_bytes.load(Ordering::Acquire))
    }

    pub fn stats(&self) -> Result<KernelHeapStats, KernelHeapError> {
        if self.state.load(Ordering::Acquire) != HEAP_INITIALIZED {
            return Err(KernelHeapError::NotInitialized);
        }
        Ok(KernelHeapStats {
            heap_bytes: self.total_pages.load(Ordering::Acquire) * KERNEL_HEAP_PAGE_SIZE,
            allocated_bytes: self.allocated_bytes.load(Ordering::Acquire),
            peak_allocated_bytes: self.peak_allocated_bytes.load(Ordering::Acquire),
            slab_allocations: self.slab_allocations.load(Ordering::Acquire),
            page_allocations: self.page_allocations.load(Ordering::Acquire),
            frees: self.frees.load(Ordering::Acquire),
            double_free_detected: self.double_free_detected.load(Ordering::Acquire) != 0,
            invalid_free_detected: self.invalid_free_detected.load(Ordering::Acquire) != 0,
            corrupt_free_list_detected: self.corrupt_free_list_detected.load(Ordering::Acquire)
                != 0,
        })
    }

    pub fn allocate_checked(&self, layout: Layout) -> Result<*mut u8, KernelHeapError> {
        if self.state.load(Ordering::Acquire) != HEAP_INITIALIZED {
            return Err(KernelHeapError::NotInitialized);
        }
        if layout.align() > KERNEL_HEAP_PAGE_SIZE {
            return Err(KernelHeapError::InvalidLayout);
        }

        let _guard = self.lock();
        if let Some(class) = class_for_layout(layout.size(), layout.align()) {
            self.allocate_slab_locked(class)
        } else {
            self.allocate_pages_locked(layout)
        }
    }

    pub fn deallocate_checked(&self, ptr: *mut u8, layout: Layout) -> Result<(), KernelHeapError> {
        let result = self.deallocate_checked_inner(ptr, layout);
        match result {
            Err(KernelHeapError::InvalidFree) => {
                self.invalid_free_detected.fetch_add(1, Ordering::AcqRel);
            }
            Err(KernelHeapError::CorruptFreeList) => {
                self.corrupt_free_list_detected
                    .fetch_add(1, Ordering::AcqRel);
            }
            _ => {}
        }
        result
    }

    fn deallocate_checked_inner(
        &self,
        ptr: *mut u8,
        layout: Layout,
    ) -> Result<(), KernelHeapError> {
        if self.state.load(Ordering::Acquire) != HEAP_INITIALIZED {
            return Err(KernelHeapError::NotInitialized);
        }
        if ptr.is_null() {
            return Err(KernelHeapError::InvalidFree);
        }

        let _guard = self.lock();
        let offset = self.offset_for_ptr(ptr as usize)?;
        let page = offset / KERNEL_HEAP_PAGE_SIZE;
        let state = self.page_state[page].load(Ordering::Acquire);
        if state == PAGE_FREE {
            self.double_free_detected.fetch_add(1, Ordering::AcqRel);
            return Err(KernelHeapError::DoubleFree);
        }
        if state >= PAGE_SLAB_BASE {
            self.deallocate_slab_locked(ptr as usize, offset, page, state, layout)
        } else {
            self.deallocate_pages_locked(offset, page, state, layout)
        }
    }

    fn reset_metadata(&self) {
        let mut index = 0usize;
        while index < SLAB_CLASS_COUNT {
            self.free_heads[index].store(FREE_LIST_EMPTY, Ordering::Release);
            index += 1;
        }
        let mut page = 0usize;
        while page < KERNEL_HEAP_PAGES {
            self.page_state[page].store(PAGE_FREE, Ordering::Release);
            self.run_pages[page].store(0, Ordering::Release);
            self.page_live_blocks[page].store(0, Ordering::Release);
            page += 1;
        }
        self.allocated_bytes.store(0, Ordering::Release);
        self.peak_allocated_bytes.store(0, Ordering::Release);
        self.slab_allocations.store(0, Ordering::Release);
        self.page_allocations.store(0, Ordering::Release);
        self.frees.store(0, Ordering::Release);
        self.double_free_detected.store(0, Ordering::Release);
        self.invalid_free_detected.store(0, Ordering::Release);
        self.corrupt_free_list_detected.store(0, Ordering::Release);
    }

    fn allocate_slab_locked(&self, class: usize) -> Result<*mut u8, KernelHeapError> {
        if self.free_heads[class].load(Ordering::Acquire) == FREE_LIST_EMPTY {
            self.populate_slab_page_locked(class)?;
        }

        let head = self.free_heads[class].load(Ordering::Acquire);
        if head == FREE_LIST_EMPTY {
            return Err(KernelHeapError::OutOfMemory);
        }
        let total_bytes = self.total_pages.load(Ordering::Acquire) * KERNEL_HEAP_PAGE_SIZE;
        let block_size = SLAB_CLASSES[class];
        let Some(offset) = decode_valid_offset(head, total_bytes, block_size) else {
            self.corrupt_free_list_detected
                .fetch_add(1, Ordering::AcqRel);
            return Err(KernelHeapError::CorruptFreeList);
        };
        let ptr = self.ptr_for_offset(offset);
        let next = read_free_next(ptr);
        self.free_heads[class].store(next, Ordering::Release);
        let page = offset / KERNEL_HEAP_PAGE_SIZE;
        self.page_live_blocks[page].fetch_add(1, Ordering::AcqRel);
        zero_heap_bytes(ptr, block_size);
        self.record_allocation(block_size);
        self.slab_allocations.fetch_add(1, Ordering::AcqRel);
        Ok(ptr)
    }

    fn populate_slab_page_locked(&self, class: usize) -> Result<(), KernelHeapError> {
        let page = self.find_free_page_locked()?;
        self.page_state[page].store(PAGE_SLAB_BASE + class, Ordering::Release);
        self.page_live_blocks[page].store(0, Ordering::Release);
        let block_size = SLAB_CLASSES[class];
        let page_offset = page * KERNEL_HEAP_PAGE_SIZE;
        let blocks = KERNEL_HEAP_PAGE_SIZE / block_size;
        let mut block = 0usize;
        while block < blocks {
            let offset = page_offset + (block * block_size);
            let next = if block + 1 < blocks {
                encode_offset(offset + block_size)
            } else {
                FREE_LIST_EMPTY
            };
            write_free_next(self.ptr_for_offset(offset), next);
            block += 1;
        }
        self.free_heads[class].store(encode_offset(page_offset), Ordering::Release);
        Ok(())
    }

    fn allocate_pages_locked(&self, layout: Layout) -> Result<*mut u8, KernelHeapError> {
        let pages = pages_for_size(layout.size()).ok_or(KernelHeapError::InvalidLayout)?;
        let start_page = self.find_free_run_locked(pages)?;
        self.page_state[start_page].store(PAGE_LARGE_HEAD, Ordering::Release);
        self.run_pages[start_page].store(pages, Ordering::Release);
        let mut page = start_page + 1;
        while page < start_page + pages {
            self.page_state[page].store(PAGE_LARGE_TAIL, Ordering::Release);
            page += 1;
        }
        let ptr = self.ptr_for_offset(start_page * KERNEL_HEAP_PAGE_SIZE);
        zero_heap_bytes(ptr, pages * KERNEL_HEAP_PAGE_SIZE);
        self.record_allocation(pages * KERNEL_HEAP_PAGE_SIZE);
        self.page_allocations.fetch_add(1, Ordering::AcqRel);
        Ok(ptr)
    }

    fn deallocate_slab_locked(
        &self,
        ptr: usize,
        offset: usize,
        page: usize,
        state: usize,
        layout: Layout,
    ) -> Result<(), KernelHeapError> {
        let class = state - PAGE_SLAB_BASE;
        if class >= SLAB_CLASS_COUNT {
            return Err(KernelHeapError::InvalidFree);
        }
        let block_size = SLAB_CLASSES[class];
        if class_for_layout(layout.size(), layout.align()) != Some(class)
            || !(offset % KERNEL_HEAP_PAGE_SIZE).is_multiple_of(block_size)
        {
            return Err(KernelHeapError::InvalidFree);
        }
        if self.free_list_contains(class, ptr)? {
            self.double_free_detected.fetch_add(1, Ordering::AcqRel);
            return Err(KernelHeapError::DoubleFree);
        }

        let live_blocks = self.page_live_blocks[page].load(Ordering::Acquire);
        if live_blocks == 0 {
            return Err(KernelHeapError::InvalidFree);
        }

        zero_heap_bytes(ptr as *mut u8, block_size);
        let head = self.free_heads[class].load(Ordering::Acquire);
        write_free_next(ptr as *mut u8, head);
        self.free_heads[class].store(encode_offset(offset), Ordering::Release);
        self.page_live_blocks[page].store(live_blocks - 1, Ordering::Release);
        self.record_free(block_size);
        if live_blocks == 1 {
            self.reclaim_slab_page_locked(class, page)?;
        }
        Ok(())
    }

    fn deallocate_pages_locked(
        &self,
        offset: usize,
        page: usize,
        state: usize,
        layout: Layout,
    ) -> Result<(), KernelHeapError> {
        if state != PAGE_LARGE_HEAD || !offset.is_multiple_of(KERNEL_HEAP_PAGE_SIZE) {
            return Err(KernelHeapError::InvalidFree);
        }
        let pages = self.run_pages[page].load(Ordering::Acquire);
        if pages == 0 || page + pages > self.total_pages.load(Ordering::Acquire) {
            return Err(KernelHeapError::InvalidFree);
        }
        if layout.align() > KERNEL_HEAP_PAGE_SIZE || pages_for_size(layout.size()) != Some(pages) {
            return Err(KernelHeapError::InvalidFree);
        }

        zero_heap_bytes(
            self.ptr_for_offset(page * KERNEL_HEAP_PAGE_SIZE),
            pages * KERNEL_HEAP_PAGE_SIZE,
        );
        let mut cursor = page;
        while cursor < page + pages {
            self.page_state[cursor].store(PAGE_FREE, Ordering::Release);
            self.run_pages[cursor].store(0, Ordering::Release);
            self.page_live_blocks[cursor].store(0, Ordering::Release);
            cursor += 1;
        }
        self.record_free(pages * KERNEL_HEAP_PAGE_SIZE);
        Ok(())
    }

    fn reclaim_slab_page_locked(&self, class: usize, page: usize) -> Result<(), KernelHeapError> {
        let page_offset = page * KERNEL_HEAP_PAGE_SIZE;
        let page_end = page_offset + KERNEL_HEAP_PAGE_SIZE;
        self.validate_free_list_locked(class)?;

        let mut old_head = self.free_heads[class].load(Ordering::Acquire);
        let mut new_head = FREE_LIST_EMPTY;
        while old_head != FREE_LIST_EMPTY {
            let offset = decode_offset(old_head).ok_or(KernelHeapError::CorruptFreeList)?;
            let ptr = self.ptr_for_offset(offset);
            let next = read_free_next(ptr);
            if offset < page_offset || offset >= page_end {
                write_free_next(ptr, new_head);
                new_head = encode_offset(offset);
            }
            old_head = next;
        }
        self.free_heads[class].store(new_head, Ordering::Release);
        self.page_live_blocks[page].store(0, Ordering::Release);
        self.page_state[page].store(PAGE_FREE, Ordering::Release);
        Ok(())
    }

    fn validate_free_list_locked(&self, class: usize) -> Result<(), KernelHeapError> {
        let block_size = SLAB_CLASSES[class];
        let total_bytes = self.total_pages.load(Ordering::Acquire) * KERNEL_HEAP_PAGE_SIZE;
        let max_blocks = total_bytes / block_size;
        let mut seen = 0usize;
        let mut cursor = self.free_heads[class].load(Ordering::Acquire);

        while cursor != FREE_LIST_EMPTY {
            if seen >= max_blocks {
                return Err(KernelHeapError::CorruptFreeList);
            }

            let offset = decode_valid_offset(cursor, total_bytes, block_size)
                .ok_or(KernelHeapError::CorruptFreeList)?;

            cursor = read_free_next(self.ptr_for_offset(offset));
            seen += 1;
        }

        Ok(())
    }

    fn find_free_page_locked(&self) -> Result<usize, KernelHeapError> {
        let total_pages = self.total_pages.load(Ordering::Acquire);
        let mut page = 0usize;
        while page < total_pages {
            if self.page_state[page].load(Ordering::Acquire) == PAGE_FREE {
                return Ok(page);
            }
            page += 1;
        }
        Err(KernelHeapError::OutOfMemory)
    }

    fn find_free_run_locked(&self, pages: usize) -> Result<usize, KernelHeapError> {
        if pages == 0 {
            return Err(KernelHeapError::InvalidLayout);
        }
        let total_pages = self.total_pages.load(Ordering::Acquire);
        if pages > total_pages {
            return Err(KernelHeapError::OutOfMemory);
        }
        let mut start = 0usize;
        while start + pages <= total_pages {
            let mut cursor = 0usize;
            while cursor < pages
                && self.page_state[start + cursor].load(Ordering::Acquire) == PAGE_FREE
            {
                cursor += 1;
            }
            if cursor == pages {
                return Ok(start);
            }
            start += cursor + 1;
        }
        Err(KernelHeapError::OutOfMemory)
    }

    fn free_list_contains(&self, class: usize, ptr: usize) -> Result<bool, KernelHeapError> {
        let total_bytes = self.total_pages.load(Ordering::Acquire) * KERNEL_HEAP_PAGE_SIZE;
        let block_size = SLAB_CLASSES[class];
        let mut head = self.free_heads[class].load(Ordering::Acquire);
        while head != FREE_LIST_EMPTY {
            let offset = decode_valid_offset(head, total_bytes, block_size)
                .ok_or(KernelHeapError::CorruptFreeList)?;
            let current = self.ptr_for_offset(offset) as usize;
            if current == ptr {
                return Ok(true);
            }
            head = read_free_next(current as *mut u8);
        }
        Ok(false)
    }
    fn offset_for_ptr(&self, ptr: usize) -> Result<usize, KernelHeapError> {
        let start = self.start.load(Ordering::Acquire);
        let total_bytes = self.total_pages.load(Ordering::Acquire) * KERNEL_HEAP_PAGE_SIZE;
        let Some(end) = start.checked_add(total_bytes) else {
            return Err(KernelHeapError::InvalidFree);
        };
        if ptr < start || ptr >= end {
            return Err(KernelHeapError::InvalidFree);
        }
        Ok(ptr - start)
    }

    fn ptr_for_offset(&self, offset: usize) -> *mut u8 {
        (self.start.load(Ordering::Acquire) + offset) as *mut u8
    }

    fn record_allocation(&self, bytes: usize) {
        let current = self.allocated_bytes.fetch_add(bytes, Ordering::AcqRel) + bytes;
        let mut peak = self.peak_allocated_bytes.load(Ordering::Acquire);
        while current > peak {
            match self.peak_allocated_bytes.compare_exchange(
                peak,
                current,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(actual) => peak = actual,
            }
        }
    }

    fn record_free(&self, bytes: usize) {
        let _ = self.allocated_bytes.fetch_sub(bytes, Ordering::AcqRel);
        self.frees.fetch_add(1, Ordering::AcqRel);
    }

    fn lock(&self) -> HeapLockGuard<'_> {
        HeapLockGuard::lock(&self.locked)
    }
}
