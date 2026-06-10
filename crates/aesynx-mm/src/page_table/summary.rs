use crate::{PageAccess, PagePrivilege};

use super::{PageTableError, PageTableMapper, PageTableMappingSummary};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn mapping_summary(&self) -> Result<PageTableMappingSummary, PageTableError> {
        let mut summary = PageTableMappingSummary::default();
        self.visit_mappings(|entry| {
            let flags = entry.mapping().flags();
            summary.total_pages = checked_increment(summary.total_pages)?;
            match flags.privilege {
                PagePrivilege::Kernel => {
                    summary.kernel_pages = checked_increment(summary.kernel_pages)?;
                }
                PagePrivilege::User => {
                    summary.user_pages = checked_increment(summary.user_pages)?;
                }
            }
            match flags.access {
                PageAccess::ReadOnly => {}
                PageAccess::ReadWrite => {
                    summary.writable_pages = checked_increment(summary.writable_pages)?;
                }
                PageAccess::ReadExecute => {
                    summary.executable_pages = checked_increment(summary.executable_pages)?;
                }
            }
            if flags.is_global() {
                summary.global_pages = checked_increment(summary.global_pages)?;
            }
            if flags.is_device_memory() {
                summary.device_pages = checked_increment(summary.device_pages)?;
            }
            Ok(())
        })?;
        Ok(summary)
    }
}

fn checked_increment(value: u64) -> Result<u64, PageTableError> {
    value.checked_add(1).ok_or(PageTableError::AddressOverflow)
}
