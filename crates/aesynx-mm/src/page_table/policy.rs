use crate::PagePrivilege;

use super::{PageTableError, PageTableMapper};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn ensure_no_user_mappings(&self) -> Result<(), PageTableError> {
        self.visit_mappings(|entry| {
            if matches!(entry.mapping().flags().privilege, PagePrivilege::User) {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn ensure_no_executable_mappings(&self) -> Result<(), PageTableError> {
        self.visit_mappings(|entry| {
            if entry.mapping().flags().access.executable() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn ensure_no_writable_mappings(&self) -> Result<(), PageTableError> {
        self.visit_mappings(|entry| {
            if entry.mapping().flags().access.writable() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })?;
        Ok(())
    }

    pub fn ensure_no_device_mappings(&self) -> Result<(), PageTableError> {
        self.visit_mappings(|entry| {
            if entry.mapping().flags().is_device_memory() {
                return Err(PageTableError::UnexpectedMappingFlags);
            }
            Ok(())
        })?;
        Ok(())
    }
}
