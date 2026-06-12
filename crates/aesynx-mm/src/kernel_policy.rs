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
    /// Constructs an unvalidated kernel virtual range descriptor.
    ///
    /// Callers must pass the enclosing [`KernelMappingPolicy`] through
    /// [`PageTableMapper::verify_kernel_mapping_policy`] before relying on
    /// page count, alignment, canonical-address, or overlap invariants.
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
    /// Constructs an unvalidated kernel mapping policy descriptor.
    ///
    /// This constructor preserves `const` construction for early boot and
    /// tests. It does not prove that ranges are non-empty, canonical, aligned,
    /// non-overlapping, or backed by the expected permissions. Callers must
    /// verify the descriptor with [`PageTableMapper::verify_kernel_mapping_policy`]
    /// before treating any policy field as trustworthy.
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
    status: KernelMappingPolicyStatus,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct KernelMappingPolicyStatus {
    text_rx: bool,
    rodata_read_only: bool,
    data_rw_nx: bool,
    reserved_heap_unmapped: bool,
    guard_page_unmapped: bool,
    null_page_unmapped: bool,
}

impl KernelMappingPolicyStatus {
    const fn new(
        text_rx: bool,
        rodata_read_only: bool,
        data_rw_nx: bool,
        reserved_heap_unmapped: bool,
        guard_page_unmapped: bool,
        null_page_unmapped: bool,
    ) -> Self {
        Self {
            text_rx,
            rodata_read_only,
            data_rw_nx,
            reserved_heap_unmapped,
            guard_page_unmapped,
            null_page_unmapped,
        }
    }
}

impl fmt::Debug for KernelMappingPolicyReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KernelMappingPolicyReport")
            .field("mapped_pages", &self.mapped_pages)
            .field("reserved_pages", &self.reserved_pages)
            .field("text_rx", &self.status.text_rx)
            .field("rodata_read_only", &self.status.rodata_read_only)
            .field("data_rw_nx", &self.status.data_rw_nx)
            .field(
                "reserved_heap_unmapped",
                &self.status.reserved_heap_unmapped,
            )
            .field("guard_page_unmapped", &self.status.guard_page_unmapped)
            .field("null_page_unmapped", &self.status.null_page_unmapped)
            .finish()
    }
}

impl KernelMappingPolicyReport {
    #[must_use]
    const fn new(
        mapped_pages: u64,
        reserved_pages: u64,
        status: KernelMappingPolicyStatus,
    ) -> Self {
        Self {
            mapped_pages,
            reserved_pages,
            status,
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
        self.status.text_rx
    }

    #[must_use]
    pub const fn rodata_read_only(self) -> bool {
        self.status.rodata_read_only
    }

    #[must_use]
    pub const fn data_rw_nx(self) -> bool {
        self.status.data_rw_nx
    }

    #[must_use]
    pub const fn reserved_heap_unmapped(self) -> bool {
        self.status.reserved_heap_unmapped
    }

    #[must_use]
    pub const fn guard_page_unmapped(self) -> bool {
        self.status.guard_page_unmapped
    }

    #[must_use]
    pub const fn null_page_unmapped(self) -> bool {
        self.status.null_page_unmapped
    }
}

impl<const TABLES: usize, const MAPPED_FRAMES: usize> PageTableMapper<TABLES, MAPPED_FRAMES> {
    pub fn verify_kernel_mapping_policy(
        &self,
        policy: KernelMappingPolicy,
    ) -> Result<KernelMappingPolicyReport, PageTableError> {
        self.audit()?;
        validate_policy_ranges(policy)?;

        let text_rx = self.verify_text_range(policy.text)?;
        let rodata_read_only = self.verify_rodata_range(policy.rodata)?;
        let data_rw_nx = self.verify_data_range(policy.data)?;
        let reserved_heap_unmapped = self.verify_reserved_heap_range(policy.reserved_heap)?;
        let guard_page_unmapped = self.verify_guard_page_range(policy.guard_page)?;
        let null_page_unmapped = self.verify_null_page_range(policy.null_page)?;

        let mapped_pages = checked_add(policy.text.pages(), policy.rodata.pages())?;
        let mapped_pages = checked_add(mapped_pages, policy.data.pages())?;
        let reserved_pages = checked_add(policy.reserved_heap.pages(), policy.guard_page.pages())?;
        let status = KernelMappingPolicyStatus::new(
            text_rx,
            rodata_read_only,
            data_rw_nx,
            reserved_heap_unmapped,
            guard_page_unmapped,
            null_page_unmapped,
        );

        Ok(KernelMappingPolicyReport::new(
            mapped_pages,
            reserved_pages,
            status,
        ))
    }

    fn verify_text_range(&self, range: KernelVirtualRange) -> Result<bool, PageTableError> {
        let flags = GenericPageFlags::kernel(PageAccess::ReadExecute);
        self.ensure_contiguous_flags(range.start(), range.pages(), flags)?;
        self.ensure_kernel_space_contiguous(range.start(), range.pages())?;
        self.ensure_write_protected_contiguous(range.start(), range.pages())?;
        self.ensure_executable_contiguous(range.start(), range.pages())?;
        self.ensure_normal_memory_contiguous(range.start(), range.pages())?;
        self.ensure_local_contiguous(range.start(), range.pages())?;
        Ok(true)
    }

    fn verify_rodata_range(&self, range: KernelVirtualRange) -> Result<bool, PageTableError> {
        let flags = GenericPageFlags::kernel(PageAccess::ReadOnly);
        self.ensure_contiguous_flags(range.start(), range.pages(), flags)?;
        self.ensure_kernel_space_contiguous(range.start(), range.pages())?;
        self.ensure_write_protected_contiguous(range.start(), range.pages())?;
        self.ensure_non_executable_contiguous(range.start(), range.pages())?;
        self.ensure_normal_memory_contiguous(range.start(), range.pages())?;
        self.ensure_local_contiguous(range.start(), range.pages())?;
        Ok(true)
    }

    fn verify_data_range(&self, range: KernelVirtualRange) -> Result<bool, PageTableError> {
        let flags = GenericPageFlags::kernel(PageAccess::ReadWrite);
        self.ensure_contiguous_flags(range.start(), range.pages(), flags)?;
        self.ensure_kernel_space_contiguous(range.start(), range.pages())?;
        self.ensure_non_executable_contiguous(range.start(), range.pages())?;
        self.ensure_normal_memory_contiguous(range.start(), range.pages())?;
        self.ensure_local_contiguous(range.start(), range.pages())?;
        Ok(true)
    }

    fn verify_reserved_heap_range(
        &self,
        range: KernelVirtualRange,
    ) -> Result<bool, PageTableError> {
        ensure_high_half_range(range)?;
        self.ensure_unmapped_contiguous(range.start(), range.pages())?;
        Ok(true)
    }

    fn verify_guard_page_range(&self, range: KernelVirtualRange) -> Result<bool, PageTableError> {
        ensure_high_half_range(range)?;
        self.ensure_unmapped_contiguous(range.start(), range.pages())?;
        Ok(true)
    }

    fn verify_null_page_range(&self, range: KernelVirtualRange) -> Result<bool, PageTableError> {
        ensure_null_page(range)?;
        self.ensure_unmapped_contiguous(range.start(), range.pages())?;
        Ok(true)
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
