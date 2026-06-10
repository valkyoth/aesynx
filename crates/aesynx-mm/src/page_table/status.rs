use super::{PageTableError, PageTableMapper, PageTableStatus};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn status_checked(&self) -> Result<PageTableStatus, PageTableError> {
        let audit = self.audit()?;
        Ok(PageTableStatus {
            total_tables: audit.total_tables(),
            used_tables: audit.used_tables(),
            mapped_pages: audit.mapped_pages(),
        })
    }
}
