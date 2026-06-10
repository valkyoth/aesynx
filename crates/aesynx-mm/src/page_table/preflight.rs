use super::{PageTableAudit, PageTableError, PageTableMapper};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn verify_kernel_address_space_candidate(&self) -> Result<PageTableAudit, PageTableError> {
        let audit = self.audit()?;
        self.root_table_checked()?;
        let status = self.status_checked()?;
        if status.total_tables() != audit.total_tables()
            || status.used_tables() != audit.used_tables()
            || status.mapped_pages() != audit.mapped_pages()
        {
            return Err(PageTableError::CorruptTable);
        }
        self.ensure_no_user_space_mappings()?;
        self.ensure_no_user_mappings()?;
        self.ensure_no_physical_aliases()?;
        Ok(audit)
    }

    pub fn verify_user_address_space_candidate(&self) -> Result<PageTableAudit, PageTableError> {
        let audit = self.audit()?;
        self.root_table_checked()?;
        let status = self.status_checked()?;
        if status.total_tables() != audit.total_tables()
            || status.used_tables() != audit.used_tables()
            || status.mapped_pages() != audit.mapped_pages()
        {
            return Err(PageTableError::CorruptTable);
        }
        self.ensure_no_kernel_space_user_mappings()?;
        self.ensure_no_user_space_kernel_mappings()?;
        self.ensure_no_global_mappings()?;
        self.ensure_no_physical_aliases()?;
        Ok(audit)
    }
}
