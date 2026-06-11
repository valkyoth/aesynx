use aesynx_abi::VirtAddr;

use crate::PagePrivilege;

use super::range::{
    VirtualSpace, add_pages_to_virt, validate_range_walk, validate_virt_range,
    validate_virtual_space,
};
use super::{PageTableError, PageTableMapper};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn ensure_kernel_mapped_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if !matches!(mapping.flags().privilege, PagePrivilege::Kernel) {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn ensure_user_mapped_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if !matches!(mapping.flags().privilege, PagePrivilege::User) {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn ensure_write_protected_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if mapping.flags().access.writable() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn ensure_non_executable_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if mapping.flags().access.executable() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn ensure_executable_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if !mapping.flags().access.executable() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn ensure_normal_memory_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if mapping.flags().is_device_memory() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn ensure_local_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if mapping.flags().is_global() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn ensure_kernel_space_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        validate_virtual_space(virt, page_count, VirtualSpace::Kernel)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if !matches!(mapping.flags().privilege, PagePrivilege::Kernel) {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            offset += 1;
        }

        Ok(())
    }

    pub fn ensure_user_space_contiguous(
        &self,
        virt: VirtAddr,
        page_count: u64,
    ) -> Result<(), PageTableError> {
        validate_virt_range(virt, page_count)?;
        validate_range_walk::<TABLES>(page_count)?;
        validate_virtual_space(virt, page_count, VirtualSpace::User)?;
        self.audit()?;

        let mut offset = 0u64;
        while offset < page_count {
            let mapping = self.mapping_for_address(add_pages_to_virt(virt, offset)?)?;
            if !matches!(mapping.flags().privilege, PagePrivilege::User) {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            offset += 1;
        }

        Ok(())
    }
}
