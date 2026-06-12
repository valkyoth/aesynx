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
    null_page_unmapped: bool,
}

impl fmt::Debug for KernelMappingPolicyReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KernelMappingPolicyReport")
            .field("mapped_pages", &self.mapped_pages)
            .field("reserved_pages", &self.reserved_pages)
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
mod tests {
    use alloc::format;

    use aesynx_abi::{PhysAddr, VirtAddr};

    use super::{KernelMappingPolicy, KernelVirtualRange};
    use crate::{GenericPageFlags, PageAccess, PageTableError, PageTableMapper};

    const TEXT: VirtAddr = VirtAddr::new(0xffff_9000_0000_0000);
    const RODATA: VirtAddr = VirtAddr::new(0xffff_9000_0000_2000);
    const DATA: VirtAddr = VirtAddr::new(0xffff_9000_0000_4000);
    const HEAP: VirtAddr = VirtAddr::new(0xffff_9000_0000_6000);
    const GUARD: VirtAddr = VirtAddr::new(0xffff_9000_0000_8000);
    const TEXT_PHYS: PhysAddr = PhysAddr::new(0x0020_0000);
    const RODATA_PHYS: PhysAddr = PhysAddr::new(0x0020_2000);
    const DATA_PHYS: PhysAddr = PhysAddr::new(0x0020_4000);

    fn policy() -> KernelMappingPolicy {
        KernelMappingPolicy::new(
            KernelVirtualRange::new(TEXT, 2),
            KernelVirtualRange::new(RODATA, 2),
            KernelVirtualRange::new(DATA, 2),
            KernelVirtualRange::new(HEAP, 2),
            KernelVirtualRange::new(GUARD, 1),
            KernelVirtualRange::new(VirtAddr::new(0), 1),
        )
    }

    fn mapper_with_policy() -> Result<PageTableMapper<8>, PageTableError> {
        let mut mapper = PageTableMapper::<8>::new()?;
        mapper.map_contiguous(
            TEXT,
            TEXT_PHYS,
            2,
            GenericPageFlags::kernel(PageAccess::ReadExecute),
        )?;
        mapper.map_contiguous(
            RODATA,
            RODATA_PHYS,
            2,
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        )?;
        mapper.map_contiguous(
            DATA,
            DATA_PHYS,
            2,
            GenericPageFlags::kernel(PageAccess::ReadWrite),
        )?;
        Ok(mapper)
    }

    #[test]
    fn kernel_mapping_policy_accepts_expected_layout() -> Result<(), PageTableError> {
        let mapper = mapper_with_policy()?;

        let report = mapper.verify_kernel_mapping_policy(policy())?;

        assert_eq!(report.mapped_pages(), 6);
        assert_eq!(report.reserved_pages(), 3);
        assert!(report.null_page_unmapped());
        Ok(())
    }

    #[test]
    fn kernel_mapping_policy_rejects_writable_text() -> Result<(), PageTableError> {
        let mut mapper = mapper_with_policy()?;
        mapper.protect_contiguous(TEXT, 2, GenericPageFlags::kernel(PageAccess::ReadWrite))?;

        assert_eq!(
            mapper.verify_kernel_mapping_policy(policy()),
            Err(PageTableError::UnexpectedMappingFlags)
        );
        Ok(())
    }

    #[test]
    fn kernel_mapping_policy_rejects_executable_data() -> Result<(), PageTableError> {
        let mut mapper = mapper_with_policy()?;
        mapper.protect_contiguous(DATA, 2, GenericPageFlags::kernel(PageAccess::ReadExecute))?;

        assert_eq!(
            mapper.verify_kernel_mapping_policy(policy()),
            Err(PageTableError::UnexpectedMappingFlags)
        );
        Ok(())
    }

    #[test]
    fn kernel_mapping_policy_rejects_mapped_guard_page() -> Result<(), PageTableError> {
        let mut mapper = mapper_with_policy()?;
        mapper.map_page(
            GUARD,
            PhysAddr::new(0x0020_8000),
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        )?;

        assert_eq!(
            mapper.verify_kernel_mapping_policy(policy()),
            Err(PageTableError::AlreadyMapped)
        );
        Ok(())
    }

    #[test]
    fn kernel_mapping_policy_rejects_mapped_null_page() -> Result<(), PageTableError> {
        let mut mapper = mapper_with_policy()?;
        mapper.map_page(
            VirtAddr::new(0),
            PhysAddr::new(0x0020_9000),
            GenericPageFlags::kernel(PageAccess::ReadOnly),
        )?;

        assert_eq!(
            mapper.verify_kernel_mapping_policy(policy()),
            Err(PageTableError::AlreadyMapped)
        );
        Ok(())
    }

    #[test]
    fn kernel_mapping_policy_rejects_overlapping_ranges() -> Result<(), PageTableError> {
        let mapper = mapper_with_policy()?;
        let overlapping = KernelMappingPolicy::new(
            KernelVirtualRange::new(TEXT, 2),
            KernelVirtualRange::new(VirtAddr::new(TEXT.get() + crate::FRAME_SIZE), 2),
            KernelVirtualRange::new(DATA, 2),
            KernelVirtualRange::new(HEAP, 2),
            KernelVirtualRange::new(GUARD, 1),
            KernelVirtualRange::new(VirtAddr::new(0), 1),
        );

        assert_eq!(
            mapper.verify_kernel_mapping_policy(overlapping),
            Err(PageTableError::UnexpectedVirtualAddressSpace)
        );
        Ok(())
    }

    #[test]
    fn kernel_virtual_range_debug_redacts_start_address() {
        let debug = format!("{:?}", KernelVirtualRange::new(TEXT, 2));

        assert!(debug.contains("KernelVirtualRange"));
        assert!(debug.contains("pages: 2"));
        assert!(!debug.contains("ffff_9000"));
    }
}
