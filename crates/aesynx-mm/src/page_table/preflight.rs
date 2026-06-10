use crate::PagePrivilege;

use super::{PageTableAudit, PageTableError, PageTableMapper};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn verify_kernel_address_space_candidate(&self) -> Result<PageTableAudit, PageTableError> {
        let audit = self.checked_candidate_audit()?;
        self.ensure_kernel_candidate_shape()?;
        self.ensure_no_physical_aliases()?;
        Ok(audit)
    }

    pub fn verify_user_address_space_candidate(&self) -> Result<PageTableAudit, PageTableError> {
        let audit = self.checked_candidate_audit()?;
        self.ensure_user_candidate_shape()?;
        self.ensure_no_physical_aliases()?;
        Ok(audit)
    }

    fn checked_candidate_audit(&self) -> Result<PageTableAudit, PageTableError> {
        let audit = self.audit()?;
        self.root_table_checked()?;
        let status = self.status_checked()?;
        if status.total_tables() != audit.total_tables()
            || status.used_tables() != audit.used_tables()
            || status.mapped_pages() != audit.mapped_pages()
        {
            return Err(PageTableError::CorruptTable);
        }
        if audit.mapped_pages() == 0 {
            return Err(PageTableError::EmptyAddressSpace);
        }
        Ok(audit)
    }

    fn ensure_kernel_candidate_shape(&self) -> Result<(), PageTableError> {
        let mut has_user_space_mapping = false;
        let mut has_user_mapping = false;
        let mut has_device_mapping = false;
        self.visit_mappings(|entry| {
            if entry.virt().get() >> 47 == 0 {
                has_user_space_mapping = true;
            }
            if matches!(entry.mapping().flags().privilege, PagePrivilege::User) {
                has_user_mapping = true;
            }
            if entry.mapping().flags().is_device_memory() {
                has_device_mapping = true;
            }
            Ok(())
        })?;
        if has_user_space_mapping {
            return Err(PageTableError::UnexpectedVirtualAddressSpace);
        }
        if has_user_mapping || has_device_mapping {
            return Err(PageTableError::UnexpectedMappingFlags);
        }
        Ok(())
    }

    fn ensure_user_candidate_shape(&self) -> Result<(), PageTableError> {
        let mut has_kernel_space_user_mapping = false;
        let mut has_user_space_kernel_mapping = false;
        let mut has_user_mapping = false;
        let mut has_device_mapping = false;
        let mut has_global_mapping = false;
        self.visit_mappings(|entry| {
            let in_user_space = entry.virt().get() >> 47 == 0;
            match (in_user_space, entry.mapping().flags().privilege) {
                (false, PagePrivilege::User) => has_kernel_space_user_mapping = true,
                (true, PagePrivilege::Kernel) => has_user_space_kernel_mapping = true,
                (true, PagePrivilege::User) => has_user_mapping = true,
                (false, PagePrivilege::Kernel) => {}
            }
            if entry.mapping().flags().is_device_memory() {
                has_device_mapping = true;
            }
            if entry.mapping().flags().is_global() {
                has_global_mapping = true;
            }
            Ok(())
        })?;
        if has_kernel_space_user_mapping || has_user_space_kernel_mapping {
            return Err(PageTableError::UnexpectedMappingFlags);
        }
        if !has_user_mapping {
            return Err(PageTableError::IncompleteAddressSpace);
        }
        if has_device_mapping || has_global_mapping {
            return Err(PageTableError::UnexpectedMappingFlags);
        }
        Ok(())
    }
}
