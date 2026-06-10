use super::{PageTableAudit, PageTableError, PageTableMapper};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn verify_kernel_address_space_candidate(&self) -> Result<PageTableAudit, PageTableError> {
        let audit = self.checked_candidate_audit()?;
        self.ensure_no_user_space_mappings()?;
        self.ensure_no_user_mappings()?;
        self.ensure_no_device_mappings()?;
        self.ensure_no_physical_aliases()?;
        Ok(audit)
    }

    pub fn verify_user_address_space_candidate(&self) -> Result<PageTableAudit, PageTableError> {
        let audit = self.checked_candidate_audit()?;
        self.ensure_no_kernel_space_user_mappings()?;
        self.ensure_no_user_space_kernel_mappings()?;
        self.ensure_user_candidate_has_user_mappings()?;
        self.ensure_no_device_mappings()?;
        self.ensure_no_global_mappings()?;
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

    fn ensure_user_candidate_has_user_mappings(&self) -> Result<(), PageTableError> {
        let mut has_user_mapping = false;
        self.visit_mappings(|entry| {
            if entry.virt().get() >> 47 == 0
                && matches!(
                    entry.mapping().flags().privilege,
                    crate::PagePrivilege::User
                )
            {
                has_user_mapping = true;
            }
            Ok(())
        })?;
        if has_user_mapping {
            Ok(())
        } else {
            Err(PageTableError::IncompleteAddressSpace)
        }
    }
}
