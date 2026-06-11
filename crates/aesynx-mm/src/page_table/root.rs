use super::{PageTableError, PageTableMapper, PageTableRoot};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn root_table_checked(&self) -> Result<PageTableRoot, PageTableError> {
        self.audit()?;
        Ok(PageTableRoot::new(0))
    }
}
