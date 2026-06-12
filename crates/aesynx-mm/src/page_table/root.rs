use super::{PageTableError, PageTableMapper, PageTableRoot};

impl<const TABLES: usize, const MAPPED_FRAMES: usize> PageTableMapper<TABLES, MAPPED_FRAMES> {
    pub fn root_table_checked(&self) -> Result<PageTableRoot, PageTableError> {
        self.audit()?;
        Ok(PageTableRoot::new(0))
    }
}
