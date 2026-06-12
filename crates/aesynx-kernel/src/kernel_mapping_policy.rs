use aesynx_abi::VirtAddr;
use aesynx_mm::{FRAME_SIZE, KernelMappingPolicy, KernelVirtualRange};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelSectionLayout {
    pub text_start: VirtAddr,
    pub text_end: VirtAddr,
    pub rodata_start: VirtAddr,
    pub rodata_end: VirtAddr,
    pub data_start: VirtAddr,
    pub data_end: VirtAddr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelMappingPlan {
    policy: KernelMappingPolicy,
    mapped_pages: u64,
    reserved_pages: u64,
    text_pages: u64,
    rodata_pages: u64,
    data_pages: u64,
}

impl KernelMappingPlan {
    pub fn from_sections(
        layout: KernelSectionLayout,
        heap_reserved_pages: u64,
        guard_pages: u64,
    ) -> Result<Self, KernelMappingPlanError> {
        let text_pages = section_pages(layout.text_start, layout.text_end)?;
        let rodata_pages = section_pages(layout.rodata_start, layout.rodata_end)?;
        let data_pages = section_pages(layout.data_start, layout.data_end)?;
        ensure_ordered_layout(layout)?;
        ensure_nonzero_reserved_pages(heap_reserved_pages, guard_pages)?;

        let heap_start = add_pages_to_virt(layout.data_start, data_pages)?;
        let guard_start = add_pages_to_virt(heap_start, heap_reserved_pages)?;
        let mapped_pages = checked_add(checked_add(text_pages, rodata_pages)?, data_pages)?;
        let reserved_pages = checked_add(heap_reserved_pages, guard_pages)?;
        let policy = KernelMappingPolicy::new(
            KernelVirtualRange::new(layout.text_start, text_pages),
            KernelVirtualRange::new(layout.rodata_start, rodata_pages),
            KernelVirtualRange::new(layout.data_start, data_pages),
            KernelVirtualRange::new(heap_start, heap_reserved_pages),
            KernelVirtualRange::new(guard_start, guard_pages),
            KernelVirtualRange::new(VirtAddr::new(0), 1),
        );

        Ok(Self {
            policy,
            mapped_pages,
            reserved_pages,
            text_pages,
            rodata_pages,
            data_pages,
        })
    }

    #[must_use]
    pub const fn policy(self) -> KernelMappingPolicy {
        self.policy
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
    pub const fn text_pages(self) -> u64 {
        self.text_pages
    }

    #[must_use]
    pub const fn rodata_pages(self) -> u64 {
        self.rodata_pages
    }

    #[must_use]
    pub const fn data_pages(self) -> u64 {
        self.data_pages
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KernelMappingPlanError {
    AddressOverflow,
    InvalidSectionLayout,
    InvalidReservedRange,
}

fn ensure_ordered_layout(layout: KernelSectionLayout) -> Result<(), KernelMappingPlanError> {
    if layout.text_start.get() >= layout.text_end.get()
        || layout.text_end.get() > layout.rodata_start.get()
        || layout.rodata_start.get() >= layout.rodata_end.get()
        || layout.rodata_end.get() > layout.data_start.get()
        || layout.data_start.get() >= layout.data_end.get()
    {
        return Err(KernelMappingPlanError::InvalidSectionLayout);
    }
    Ok(())
}

fn ensure_nonzero_reserved_pages(
    heap_reserved_pages: u64,
    guard_pages: u64,
) -> Result<(), KernelMappingPlanError> {
    if heap_reserved_pages == 0 || guard_pages == 0 {
        return Err(KernelMappingPlanError::InvalidReservedRange);
    }
    Ok(())
}

fn section_pages(start: VirtAddr, end: VirtAddr) -> Result<u64, KernelMappingPlanError> {
    if start.get() >= end.get() || !page_aligned(start.get()) || !page_aligned(end.get()) {
        return Err(KernelMappingPlanError::InvalidSectionLayout);
    }

    Ok((end.get() - start.get()) / FRAME_SIZE)
}

const fn page_aligned(value: u64) -> bool {
    value & (FRAME_SIZE - 1) == 0
}

fn add_pages_to_virt(virt: VirtAddr, pages: u64) -> Result<VirtAddr, KernelMappingPlanError> {
    let offset = pages
        .checked_mul(FRAME_SIZE)
        .ok_or(KernelMappingPlanError::AddressOverflow)?;
    virt.get()
        .checked_add(offset)
        .map(VirtAddr::new)
        .ok_or(KernelMappingPlanError::AddressOverflow)
}

fn checked_add(left: u64, right: u64) -> Result<u64, KernelMappingPlanError> {
    left.checked_add(right)
        .ok_or(KernelMappingPlanError::AddressOverflow)
}

#[cfg(test)]
mod tests {
    use aesynx_abi::VirtAddr;

    use super::{KernelMappingPlan, KernelMappingPlanError, KernelSectionLayout};

    const TEXT: VirtAddr = VirtAddr::new(0xffff_9000_0000_0000);
    const RODATA: VirtAddr = VirtAddr::new(0xffff_9000_0000_4000);
    const DATA: VirtAddr = VirtAddr::new(0xffff_9000_0000_6000);
    const END: VirtAddr = VirtAddr::new(0xffff_9000_0000_d000);

    fn layout() -> KernelSectionLayout {
        KernelSectionLayout {
            text_start: TEXT,
            text_end: RODATA,
            rodata_start: RODATA,
            rodata_end: DATA,
            data_start: DATA,
            data_end: END,
        }
    }

    #[test]
    fn plan_derives_section_and_reserved_page_counts() -> Result<(), KernelMappingPlanError> {
        let plan = KernelMappingPlan::from_sections(layout(), 2, 1)?;
        let policy = plan.policy();

        assert_eq!(plan.text_pages(), 4);
        assert_eq!(plan.rodata_pages(), 2);
        assert_eq!(plan.data_pages(), 7);
        assert_eq!(plan.mapped_pages(), 13);
        assert_eq!(plan.reserved_pages(), 3);
        assert_eq!(policy.text().start(), TEXT);
        assert_eq!(policy.text().pages(), 4);
        assert_eq!(policy.rodata().start(), RODATA);
        assert_eq!(policy.data().start(), DATA);
        assert_eq!(policy.reserved_heap().start(), END);
        assert_eq!(policy.reserved_heap().pages(), 2);
        assert_eq!(
            policy.guard_page().start(),
            VirtAddr::new(END.get() + 2 * aesynx_mm::FRAME_SIZE)
        );
        assert_eq!(policy.guard_page().pages(), 1);
        assert_eq!(policy.null_page().start(), VirtAddr::new(0));
        assert_eq!(policy.null_page().pages(), 1);
        Ok(())
    }

    #[test]
    fn plan_rejects_unaligned_sections() {
        let mut invalid = layout();
        invalid.rodata_start = VirtAddr::new(RODATA.get() + 1);

        assert_eq!(
            KernelMappingPlan::from_sections(invalid, 2, 1),
            Err(KernelMappingPlanError::InvalidSectionLayout)
        );
    }

    #[test]
    fn plan_rejects_overlapping_sections() {
        let mut invalid = layout();
        invalid.rodata_start = VirtAddr::new(TEXT.get() + aesynx_mm::FRAME_SIZE);

        assert_eq!(
            KernelMappingPlan::from_sections(invalid, 2, 1),
            Err(KernelMappingPlanError::InvalidSectionLayout)
        );
    }

    #[test]
    fn plan_rejects_empty_reserved_ranges() {
        assert_eq!(
            KernelMappingPlan::from_sections(layout(), 0, 1),
            Err(KernelMappingPlanError::InvalidReservedRange)
        );
        assert_eq!(
            KernelMappingPlan::from_sections(layout(), 2, 0),
            Err(KernelMappingPlanError::InvalidReservedRange)
        );
    }

    #[test]
    fn plan_rejects_address_overflow() {
        assert_eq!(
            KernelMappingPlan::from_sections(layout(), u64::MAX / aesynx_mm::FRAME_SIZE, 1),
            Err(KernelMappingPlanError::AddressOverflow)
        );
    }
}
