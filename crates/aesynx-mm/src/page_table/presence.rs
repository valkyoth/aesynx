use aesynx_abi::VirtAddr;

use super::{PageTableError, PageTableMapper};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn is_page_mapped(&self, virt: VirtAddr) -> Result<bool, PageTableError> {
        match self.mapping_for_page(virt) {
            Ok(_mapping) => Ok(true),
            Err(PageTableError::NotMapped) => Ok(false),
            Err(error) => Err(error),
        }
    }
}
