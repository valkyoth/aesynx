use core::fmt;

use aesynx_abi::{PhysAddr, VirtAddr};

use crate::FRAME_SIZE;

use super::address::{PAGE_OFFSET_MASK, is_canonical, validate_virt_page};
use super::range::{add_pages_to_phys, add_pages_to_virt, validate_range_walk};
use super::{PageTableError, PageTableMapper, TranslatedRange};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn translate_contiguous_range_checked(
        &self,
        virt: VirtAddr,
        byte_len: u64,
    ) -> Result<TranslatedRange, PageTableError> {
        let checked = validate_virt_byte_range(virt, byte_len)?;
        validate_range_walk::<TABLES>(checked.pages)?;
        self.audit()?;

        let first = self.mapping_for_address(checked.start_page)?;
        let offset = virt.get() & PAGE_OFFSET_MASK;
        let start_phys = first
            .phys()
            .get()
            .checked_add(offset)
            .map(PhysAddr::new)
            .ok_or(PageTableError::AddressOverflow)?;

        let mut page_offset = 1u64;
        while page_offset < checked.pages {
            let mapping =
                self.mapping_for_address(add_pages_to_virt(checked.start_page, page_offset)?)?;
            if mapping.phys() != add_pages_to_phys(first.phys(), page_offset)? {
                return Err(PageTableError::NonContiguousRange);
            }
            if mapping.flags() != first.flags() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            page_offset += 1;
        }

        Ok(TranslatedRange::new(
            start_phys,
            byte_len,
            checked.pages,
            first.flags(),
        ))
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct ValidatedByteRange {
    start_page: VirtAddr,
    pages: u64,
}

impl fmt::Debug for ValidatedByteRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ValidatedByteRange")
            .field("start_page", &"<redacted>")
            .field("pages", &self.pages)
            .finish()
    }
}

fn validate_virt_byte_range(
    virt: VirtAddr,
    byte_len: u64,
) -> Result<ValidatedByteRange, PageTableError> {
    if byte_len == 0 {
        return Err(PageTableError::InvalidByteCount);
    }
    if !is_canonical(virt.get()) {
        return Err(PageTableError::InvalidVirtualAddress);
    }

    let last_byte = virt
        .get()
        .checked_add(byte_len - 1)
        .ok_or(PageTableError::AddressOverflow)?;
    if !is_canonical(last_byte) {
        return Err(PageTableError::InvalidVirtualAddress);
    }
    if canonical_sign_bit(virt) != canonical_sign_bit(VirtAddr::new(last_byte)) {
        return Err(PageTableError::InvalidVirtualAddress);
    }

    let start_page = VirtAddr::new(virt.get() & !PAGE_OFFSET_MASK);
    let end_page = VirtAddr::new(last_byte & !PAGE_OFFSET_MASK);
    validate_virt_page(start_page)?;
    validate_virt_page(end_page)?;

    let pages = end_page
        .get()
        .checked_sub(start_page.get())
        .ok_or(PageTableError::AddressOverflow)?
        .checked_div(FRAME_SIZE)
        .and_then(|pages_before_last| pages_before_last.checked_add(1))
        .ok_or(PageTableError::AddressOverflow)?;

    Ok(ValidatedByteRange { start_page, pages })
}

fn canonical_sign_bit(virt: VirtAddr) -> u64 {
    (virt.get() >> 47) & 1
}

#[cfg(test)]
mod tests {
    use alloc::{format, string::ToString};

    use aesynx_abi::VirtAddr;

    use crate::page_table::PageTableError;

    use super::{ValidatedByteRange, validate_virt_byte_range};

    #[test]
    fn validated_byte_range_debug_redacts_start_page() -> Result<(), PageTableError> {
        let virt = VirtAddr::new(0xffff_8000_0000_3000);
        let range = validate_virt_byte_range(virt, 64)?;
        let debug = format!("{range:?}");

        assert!(debug.contains("<redacted>"));
        assert!(debug.contains("pages"));
        assert!(!debug.contains("VirtAddr"));
        assert!(!debug.contains(&virt.get().to_string()));
        Ok(())
    }

    #[test]
    fn validated_byte_range_debug_keeps_accounting_visible() {
        let range = ValidatedByteRange {
            start_page: VirtAddr::new(0),
            pages: 3,
        };
        let debug = format!("{range:?}");

        assert!(debug.contains("<redacted>"));
        assert!(debug.contains("pages"));
        assert!(debug.contains('3'));
        assert!(!debug.contains("VirtAddr"));
    }
}
