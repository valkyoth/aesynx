use aesynx_abi::VirtAddr;

use super::{PAGE_TABLE_ENTRIES, PageTableError, PageTableMapper, PageTableMapping, PageTableSlot};

impl<const TABLES: usize> PageTableMapper<TABLES> {
    pub fn visit_mappings<F>(&self, mut visitor: F) -> Result<u64, PageTableError>
    where
        F: FnMut(PageTableMapping) -> Result<(), PageTableError>,
    {
        if TABLES == 0 {
            return Err(PageTableError::EmptyArena);
        }
        if !self.used[0] {
            return Err(PageTableError::CorruptTable);
        }

        let mut count = 0u64;
        let mut l0 = 0usize;
        while l0 < PAGE_TABLE_ENTRIES {
            let table_1 = match self.child_table_index(self.tables[0].slots[l0])? {
                Some(index) => index,
                None => {
                    l0 += 1;
                    continue;
                }
            };

            let mut l1 = 0usize;
            while l1 < PAGE_TABLE_ENTRIES {
                let table_2 = match self.child_table_index(self.tables[table_1].slots[l1])? {
                    Some(index) => index,
                    None => {
                        l1 += 1;
                        continue;
                    }
                };

                let mut l2 = 0usize;
                while l2 < PAGE_TABLE_ENTRIES {
                    let table_3 = match self.child_table_index(self.tables[table_2].slots[l2])? {
                        Some(index) => index,
                        None => {
                            l2 += 1;
                            continue;
                        }
                    };

                    let mut l3 = 0usize;
                    while l3 < PAGE_TABLE_ENTRIES {
                        let slot = self.tables[table_3].slots[l3];
                        if slot.is_next() {
                            return Err(PageTableError::CorruptTable);
                        }
                        if !slot.is_empty() {
                            let mapping = slot.mapping().ok_or(PageTableError::CorruptTable)?;
                            visitor(PageTableMapping::new(
                                virt_from_indices(l0, l1, l2, l3),
                                mapping,
                            ))?;
                            count = count
                                .checked_add(1)
                                .ok_or(PageTableError::AddressOverflow)?;
                        }
                        l3 += 1;
                    }
                    l2 += 1;
                }
                l1 += 1;
            }
            l0 += 1;
        }

        if count != self.mapped_pages {
            return Err(PageTableError::CorruptTable);
        }

        Ok(count)
    }

    fn child_table_index(&self, slot: PageTableSlot) -> Result<Option<usize>, PageTableError> {
        if slot.is_empty() {
            return Ok(None);
        }
        if !slot.is_next() {
            return Err(PageTableError::CorruptTable);
        }
        let next = slot.next_index()?;
        if next >= TABLES || !self.used[next] {
            return Err(PageTableError::CorruptTable);
        }
        Ok(Some(next))
    }
}

fn virt_from_indices(l0: usize, l1: usize, l2: usize, l3: usize) -> VirtAddr {
    let raw = ((l0 as u64) << 39) | ((l1 as u64) << 30) | ((l2 as u64) << 21) | ((l3 as u64) << 12);
    if l0 & 0x100 == 0 {
        VirtAddr::new(raw)
    } else {
        VirtAddr::new(raw | 0xffff_0000_0000_0000)
    }
}
