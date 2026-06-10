use super::{PAGE_TABLE_ENTRIES, PageTableAudit, PageTableError, PageTableMapper};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn audit(&self) -> Result<PageTableAudit, PageTableError> {
        if TABLES == 0 {
            return Err(PageTableError::EmptyArena);
        }
        if !self.used[0] {
            return Err(PageTableError::CorruptTable);
        }

        let mut reachable = [false; TABLES];
        let mut levels = [0usize; TABLES];
        reachable[0] = true;

        let mut changed = true;
        while changed {
            changed = false;
            let mut table_index = 0usize;
            while table_index < TABLES {
                if reachable[table_index] {
                    let mut slot_index = 0usize;
                    while slot_index < PAGE_TABLE_ENTRIES {
                        let slot = self.tables[table_index].slots[slot_index];
                        if slot.is_next() {
                            if levels[table_index] + 1 >= super::PAGE_TABLE_LEVELS {
                                return Err(PageTableError::CorruptTable);
                            }
                            let next = slot.next_index()?;
                            if next >= TABLES || !self.used[next] {
                                return Err(PageTableError::CorruptTable);
                            }
                            let next_level = levels[table_index] + 1;
                            if reachable[next] && levels[next] != next_level {
                                return Err(PageTableError::CorruptTable);
                            }
                            levels[next] = next_level;
                            if !reachable[next] {
                                reachable[next] = true;
                                changed = true;
                            }
                        } else if !slot.is_empty()
                            && levels[table_index] != super::PAGE_TABLE_LEVELS - 1
                        {
                            return Err(PageTableError::CorruptTable);
                        }
                        slot_index += 1;
                    }
                }
                table_index += 1;
            }
        }

        let mut incoming = [0u8; TABLES];
        let mut used_tables = 0u64;
        let mut reachable_tables = 0u64;
        let mut mapped_pages = 0u64;
        let mut table_index = 0usize;
        while table_index < TABLES {
            if self.used[table_index] {
                used_tables += 1;
                if !reachable[table_index] {
                    return Err(PageTableError::CorruptTable);
                }
            } else if !self.tables[table_index].is_empty() {
                return Err(PageTableError::CorruptTable);
            }

            if reachable[table_index] {
                reachable_tables += 1;
                let mut slot_index = 0usize;
                while slot_index < PAGE_TABLE_ENTRIES {
                    let slot = self.tables[table_index].slots[slot_index];
                    if slot.is_next() {
                        if levels[table_index] + 1 >= super::PAGE_TABLE_LEVELS {
                            return Err(PageTableError::CorruptTable);
                        }
                        let next = slot.next_index()?;
                        if next >= TABLES || !self.used[next] || incoming[next] != 0 {
                            return Err(PageTableError::CorruptTable);
                        }
                        incoming[next] = 1;
                    } else if !slot.is_empty() {
                        if levels[table_index] != super::PAGE_TABLE_LEVELS - 1 {
                            return Err(PageTableError::CorruptTable);
                        }
                        if slot.mapping().is_none() {
                            return Err(PageTableError::CorruptTable);
                        }
                        mapped_pages = mapped_pages
                            .checked_add(1)
                            .ok_or(PageTableError::AddressOverflow)?;
                    }
                    slot_index += 1;
                }
            }

            table_index += 1;
        }

        if incoming[0] != 0 {
            return Err(PageTableError::CorruptTable);
        }

        if mapped_pages != self.mapped_pages {
            return Err(PageTableError::CorruptTable);
        }

        Ok(PageTableAudit::new(
            TABLES as u64,
            used_tables,
            reachable_tables,
            mapped_pages,
        ))
    }
}
