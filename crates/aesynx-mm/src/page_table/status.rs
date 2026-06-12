use super::{PageTableError, PageTableMapper, PageTableStatus};

impl<const TABLES: usize, const MAPPED_FRAMES: usize> PageTableMapper<TABLES, MAPPED_FRAMES> {
    pub fn status_checked(&self) -> Result<PageTableStatus, PageTableError> {
        let audit = self.audit()?;
        Ok(PageTableStatus::new(
            audit.total_tables(),
            audit.used_tables(),
            audit.mapped_pages(),
        ))
    }
}
