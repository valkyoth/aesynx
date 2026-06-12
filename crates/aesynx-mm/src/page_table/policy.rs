use crate::PagePrivilege;

use super::{PageTableError, PageTableMapper};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn ensure_no_user_space_mappings(&self) -> Result<(), PageTableError> {
        self.visit_policy_mappings(|entry| {
            if entry.virt().get() >> 47 == 0 {
                return Err(PageTableError::UnexpectedVirtualAddressSpace);
            }
            Ok(())
        })
    }

    pub fn ensure_no_user_mappings(&self) -> Result<(), PageTableError> {
        self.visit_policy_mappings(|entry| {
            if matches!(entry.mapping().flags().privilege(), PagePrivilege::User) {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })
    }

    pub fn ensure_no_kernel_space_user_mappings(&self) -> Result<(), PageTableError> {
        self.visit_policy_mappings(|entry| {
            if entry.virt().get() >> 47 != 0
                && matches!(entry.mapping().flags().privilege(), PagePrivilege::User)
            {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })
    }

    pub fn ensure_no_user_space_kernel_mappings(&self) -> Result<(), PageTableError> {
        self.visit_policy_mappings(|entry| {
            if entry.virt().get() >> 47 == 0
                && matches!(entry.mapping().flags().privilege(), PagePrivilege::Kernel)
            {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })
    }

    pub fn ensure_no_executable_mappings(&self) -> Result<(), PageTableError> {
        self.visit_policy_mappings(|entry| {
            if entry.mapping().flags().access().executable() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })
    }

    pub fn ensure_no_writable_mappings(&self) -> Result<(), PageTableError> {
        self.visit_policy_mappings(|entry| {
            if entry.mapping().flags().access().writable() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })
    }

    pub fn ensure_no_device_mappings(&self) -> Result<(), PageTableError> {
        self.visit_policy_mappings(|entry| {
            if entry.mapping().flags().is_device_memory() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })
    }

    pub fn ensure_no_global_mappings(&self) -> Result<(), PageTableError> {
        self.visit_policy_mappings(|entry| {
            if entry.mapping().flags().is_global() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })
    }

    pub fn ensure_no_physical_aliases(&self) -> Result<(), PageTableError> {
        self.audit()?;
        Ok(())
    }

    fn visit_policy_mappings<F>(&self, visitor: F) -> Result<(), PageTableError>
    where
        F: FnMut(super::PageTableMapping) -> Result<(), PageTableError>,
    {
        self.audit()?;
        self.visit_mappings(visitor)?;
        Ok(())
    }
}
