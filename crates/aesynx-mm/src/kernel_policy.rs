use core::fmt;

use aesynx_abi::VirtAddr;

use crate::{FRAME_SIZE, GenericPageFlags, PageAccess, PageTableError, PageTableMapper};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct KernelVirtualRange {
    start: VirtAddr,
    pages: u64,
}

impl fmt::Debug for KernelVirtualRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KernelVirtualRange")
            .field("start", &"<redacted>")
            .field("pages", &self.pages)
            .finish()
    }
}

impl KernelVirtualRange {
    #[must_use]
    pub const fn new(start: VirtAddr, pages: u64) -> Self {
        Self { start, pages }
    }

    #[must_use]
    pub const fn start(self) -> VirtAddr {
        self.start
    }

    #[must_use]
    pub const fn pages(self) -> u64 {
        self.pages
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelMappingPolicy {
    text: KernelVirtualRange,
    rodata: KernelVirtualRange,
    data: KernelVirtualRange,
    reserved_heap: KernelVirtualRange,
    guard_page: KernelVirtualRange,
    null_page: KernelVirtualRange,
}

impl KernelMappingPolicy {
    #[must_use]
    pub const fn new(
        text: KernelVirtualRange,
        rodata: KernelVirtualRange,
        data: KernelVirtualRange,
        reserved_heap: KernelVirtualRange,
        guard_page: KernelVirtualRange,
        null_page: KernelVirtualRange,
    ) -> Self {
        Self {
            text,
            rodata,
            data,
            reserved_heap,
            guard_page,
            null_page,
        }
    }

    #[must_use]
    pub const fn text(self) -> KernelVirtualRange {
        self.text
    }

    #[must_use]
    pub const fn rodata(self) -> KernelVirtualRange {
        self.rodata
    }

    #[must_use]
    pub const fn data(self) -> KernelVirtualRange {
        self.data
    }

    #[must_use]
    pub const fn reserved_heap(self) -> KernelVirtualRange {
        self.reserved_heap
    }

    #[must_use]
    pub const fn guard_page(self) -> KernelVirtualRange {
        self.guard_page
    }

    #[must_use]
    pub const fn null_page(self) -> KernelVirtualRange {
        self.null_page
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct KernelMappingPolicyReport {
    mapped_pages: u64,
    reserved_pages: u64,
    text_rx: bool,
    rodata_read_only: bool,
    data_rw_nx: bool,
    reserved_heap_unmapped: bool,
    guard_page_unmapped: bool,
    null_page_unmapped: bool,
}

impl fmt::Debug for KernelMappingPolicyReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KernelMappingPolicyReport")
            .field("mapped_pages", &self.mapped_pages)
            .field("reserved_pages", &self.reserved_pages)
            .field("text_rx", &self.text_rx)
            .field("rodata_read_only", &self.rodata_read_only)
            .field("data_rw_nx", &self.data_rw_nx)
            .field("reserved_heap_unmapped", &self.reserved_heap_unmapped)
            .field("guard_page_unmapped", &self.guard_page_unmapped)
            .field("null_page_unmapped", &self.null_page_unmapped)
            .finish()
    }
}

impl KernelMappingPolicyReport {
    #[must_use]
    pub(crate) const fn new(mapped_pages: u64, reserved_pages: u64) -> Self {
        Self {
            mapped_pages,
            reserved_pages,
            text_rx: true,
            rodata_read_only: true,
            data_rw_nx: true,
            reserved_heap_unmapped: true,
            guard_page_unmapped: true,
            null_page_unmapped: true,
        }
    }

    #[must_use]
    pub const fn mapped_pages(self) -> u64 {
        self.mapped_pages
    }

    #[must_use]
    pub const fn reserved_pages(self) -> u64 {
        self.reserved_pages
    }

    #[must_use]
    pub const fn text_rx(self) -> bool {
        self.text_rx
    }

    #[must_use]
    pub const fn rodata_read_only(self) -> bool {
        self.rodata_read_only
    }

    #[must_use]
    pub const fn data_rw_nx(self) -> bool {
        self.data_rw_nx
    }

    #[must_use]
    pub const fn reserved_heap_unmapped(self) -> bool {
        self.reserved_heap_unmapped
    }

    #[must_use]
    pub const fn guard_page_unmapped(self) -> bool {
        self.guard_page_unmapped
    }

    #[must_use]
    pub const fn null_page_unmapped(self) -> bool {
        self.null_page_unmapped
    }
}

impl<const TABLES: usize, const MAPPED_FRAMES: usize> PageTableMapper<TABLES, MAPPED_FRAMES> {
    pub fn verify_kernel_mapping_policy(
        &self,
        policy: KernelMappingPolicy,
    ) -> Result<KernelMappingPolicyReport, PageTableError> {
        self.audit()?;
        validate_policy_ranges(policy)?;

        let text_flags = GenericPageFlags::kernel(PageAccess::ReadExecute);
        self.ensure_contiguous_flags(policy.text.start(), policy.text.pages(), text_flags)?;
        self.ensure_kernel_space_contiguous(policy.text.start(), policy.text.pages())?;
        self.ensure_write_protected_contiguous(policy.text.start(), policy.text.pages())?;
        self.ensure_executable_contiguous(policy.text.start(), policy.text.pages())?;
        self.ensure_normal_memory_contiguous(policy.text.start(), policy.text.pages())?;
        self.ensure_local_contiguous(policy.text.start(), policy.text.pages())?;

        let rodata_flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
        self.ensure_contiguous_flags(policy.rodata.start(), policy.rodata.pages(), rodata_flags)?;
        self.ensure_kernel_space_contiguous(policy.rodata.start(), policy.rodata.pages())?;
        self.ensure_write_protected_contiguous(policy.rodata.start(), policy.rodata.pages())?;
        self.ensure_non_executable_contiguous(policy.rodata.start(), policy.rodata.pages())?;
        self.ensure_normal_memory_contiguous(policy.rodata.start(), policy.rodata.pages())?;
        self.ensure_local_contiguous(policy.rodata.start(), policy.rodata.pages())?;

        let data_flags = GenericPageFlags::kernel(PageAccess::ReadWrite);
        self.ensure_contiguous_flags(policy.data.start(), policy.data.pages(), data_flags)?;
        self.ensure_kernel_space_contiguous(policy.data.start(), policy.data.pages())?;
        self.ensure_non_executable_contiguous(policy.data.start(), policy.data.pages())?;
        self.ensure_normal_memory_contiguous(policy.data.start(), policy.data.pages())?;
        self.ensure_local_contiguous(policy.data.start(), policy.data.pages())?;

        ensure_high_half_range(policy.reserved_heap)?;
        self.ensure_unmapped_contiguous(
            policy.reserved_heap.start(),
            policy.reserved_heap.pages(),
        )?;
        ensure_high_half_range(policy.guard_page)?;
        self.ensure_unmapped_contiguous(policy.guard_page.start(), policy.guard_page.pages())?;
        ensure_null_page(policy.null_page)?;
        self.ensure_unmapped_contiguous(policy.null_page.start(), policy.null_page.pages())?;

        let mapped_pages = checked_add(policy.text.pages(), policy.rodata.pages())?;
        let mapped_pages = checked_add(mapped_pages, policy.data.pages())?;
        let reserved_pages = checked_add(policy.reserved_heap.pages(), policy.guard_page.pages())?;

        Ok(KernelMappingPolicyReport::new(mapped_pages, reserved_pages))
    }
}

fn validate_policy_ranges(policy: KernelMappingPolicy) -> Result<(), PageTableError> {
    let ranges = [
        policy.text,
        policy.rodata,
        policy.data,
        policy.reserved_heap,
        policy.guard_page,
        policy.null_page,
    ];

    let mut left = 0usize;
    while left < ranges.len() {
        let _ = range_end_exclusive(ranges[left])?;
        let mut right = left + 1;
        while right < ranges.len() {
            if ranges_overlap(ranges[left], ranges[right])? {
                return Err(PageTableError::UnexpectedVirtualAddressSpace);
            }
            right += 1;
        }
        left += 1;
    }
    Ok(())
}

fn ensure_high_half_range(range: KernelVirtualRange) -> Result<(), PageTableError> {
    let start = range.start().get();
    let end = range_end_exclusive(range)?;
    if start >> 47 == 0 || (end - 1) >> 47 == 0 {
        return Err(PageTableError::UnexpectedVirtualAddressSpace);
    }
    Ok(())
}

fn ensure_null_page(range: KernelVirtualRange) -> Result<(), PageTableError> {
    if range.start().get() != 0 || range.pages() != 1 {
        return Err(PageTableError::UnexpectedVirtualAddressSpace);
    }
    Ok(())
}

fn ranges_overlap(
    left: KernelVirtualRange,
    right: KernelVirtualRange,
) -> Result<bool, PageTableError> {
    let left_start = left.start().get();
    let left_end = range_end_exclusive(left)?;
    let right_start = right.start().get();
    let right_end = range_end_exclusive(right)?;

    Ok(left_start < right_end && right_start < left_end)
}

fn range_end_exclusive(range: KernelVirtualRange) -> Result<u64, PageTableError> {
    if range.pages() == 0 {
        return Err(PageTableError::InvalidPageCount);
    }
    let bytes = range
        .pages()
        .checked_mul(FRAME_SIZE)
        .ok_or(PageTableError::AddressOverflow)?;
    range
        .start()
        .get()
        .checked_add(bytes)
        .ok_or(PageTableError::AddressOverflow)
}

fn checked_add(left: u64, right: u64) -> Result<u64, PageTableError> {
    left.checked_add(right)
        .ok_or(PageTableError::AddressOverflow)
}

#[cfg(test)]
mod tests;
