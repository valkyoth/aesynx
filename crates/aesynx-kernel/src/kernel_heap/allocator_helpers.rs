use core::sync::atomic::Ordering;

use super::allocator::KernelHeapAllocator;
use super::free_list::{FREE_LIST_EMPTY, decode_valid_offset, read_free_next};
use super::layout::{
    KERNEL_HEAP_PAGE_SIZE, KERNEL_HEAP_PAGES, PAGE_FREE, PAGE_SLAB_BASE, SLAB_CLASS_COUNT,
    SLAB_CLASSES,
};
use super::lock::HeapLockGuard;
use super::stats::KernelHeapError;

impl KernelHeapAllocator {
    pub(super) fn reset_metadata(&self) {
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

    pub(super) fn find_free_page_locked(&self) -> Result<usize, KernelHeapError> {
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

    pub(super) fn find_free_run_locked(&self, pages: usize) -> Result<usize, KernelHeapError> {
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

    pub(super) fn free_list_contains(
        &self,
        class: usize,
        ptr: usize,
    ) -> Result<bool, KernelHeapError> {
        let total_bytes = self.total_pages.load(Ordering::Acquire) * KERNEL_HEAP_PAGE_SIZE;
        let block_size = SLAB_CLASSES[class];
        let max_blocks = self.free_list_step_limit_locked(class);
        let mut head = self.free_heads[class].load(Ordering::Acquire);
        let mut seen = 0usize;
        while head != FREE_LIST_EMPTY {
            if seen >= max_blocks {
                return Err(KernelHeapError::CorruptFreeList);
            }
            let offset = decode_valid_offset(head, total_bytes, block_size)
                .ok_or(KernelHeapError::CorruptFreeList)?;
            let current = self.ptr_for_offset(offset) as usize;
            if current == ptr {
                return Ok(true);
            }
            head = read_free_next(current as *mut u8);
            seen += 1;
        }
        Ok(false)
    }

    pub(super) fn free_list_step_limit_locked(&self, class: usize) -> usize {
        let total_pages = self.total_pages.load(Ordering::Acquire);
        let page_state = PAGE_SLAB_BASE + class;
        let blocks_per_page = KERNEL_HEAP_PAGE_SIZE / SLAB_CLASSES[class];
        let mut slab_pages = 0usize;
        let mut page = 0usize;
        while page < total_pages {
            if self.page_state[page].load(Ordering::Acquire) == page_state {
                slab_pages += 1;
            }
            page += 1;
        }
        slab_pages * blocks_per_page
    }

    pub(super) fn offset_for_ptr(&self, ptr: usize) -> Result<usize, KernelHeapError> {
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

    pub(super) fn ptr_for_offset(&self, offset: usize) -> *mut u8 {
        (self.start.load(Ordering::Acquire) + offset) as *mut u8
    }

    pub(super) fn record_allocation(&self, bytes: usize) {
        let mut current = self.allocated_bytes.load(Ordering::Acquire);
        let current = loop {
            let Some(next) = current.checked_add(bytes) else {
                self.corrupt_free_list_detected
                    .fetch_add(1, Ordering::AcqRel);
                return;
            };
            match self.allocated_bytes.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break next,
                Err(actual) => current = actual,
            }
        };
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

    pub(super) fn record_free(&self, bytes: usize) {
        let mut current = self.allocated_bytes.load(Ordering::Acquire);
        loop {
            let Some(next) = current.checked_sub(bytes) else {
                self.invalid_free_detected.fetch_add(1, Ordering::AcqRel);
                return;
            };
            match self.allocated_bytes.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
        self.frees.fetch_add(1, Ordering::AcqRel);
    }

    pub(super) fn lock(&self) -> Result<HeapLockGuard<'_>, KernelHeapError> {
        HeapLockGuard::lock(&self.locked).map_err(KernelHeapError::from)
    }

    #[cfg(test)]
    pub(crate) fn offset_for_ptr_for_test(&self, ptr: *mut u8) -> Result<usize, KernelHeapError> {
        self.offset_for_ptr(ptr as usize)
    }
}
