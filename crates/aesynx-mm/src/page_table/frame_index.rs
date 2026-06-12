use aesynx_abi::PhysAddr;

use super::PageTableError;

pub(crate) const DEFAULT_MAPPED_FRAME_INDEX_ENTRIES: usize = 64;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct MappedFrameIndex<const ENTRIES: usize> {
    frames: [MappedFrameIndexEntry; ENTRIES],
    len: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct MappedFrameInsert {
    position: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct MappedFrameRemove {
    position: usize,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct MappedFrameIndexEntry {
    phys: u64,
}

impl<const ENTRIES: usize> MappedFrameIndex<ENTRIES> {
    pub(crate) const fn empty() -> Self {
        Self {
            frames: [MappedFrameIndexEntry::EMPTY; ENTRIES],
            len: 0,
        }
    }

    pub(crate) fn validate_insert(
        &self,
        phys: PhysAddr,
    ) -> Result<MappedFrameInsert, PageTableError> {
        self.validate_len(self.len as u64)?;
        if self.len == self.capacity()? {
            return Err(PageTableError::OutOfPageTables);
        }
        match self.search(phys.get())? {
            Ok(_position) => Err(PageTableError::PhysicalAlias),
            Err(position) => Ok(MappedFrameInsert { position }),
        }
    }

    pub(crate) fn insert_validated(
        &mut self,
        insert: MappedFrameInsert,
        phys: PhysAddr,
    ) -> Result<(), PageTableError> {
        if self.len == self.capacity()? || insert.position > self.len {
            return Err(PageTableError::CorruptTable);
        }

        let mut index = self.len;
        while index > insert.position {
            self.set(index, self.get(index - 1)?)?;
            index -= 1;
        }
        self.set(insert.position, MappedFrameIndexEntry::new(phys))?;
        self.len += 1;
        Ok(())
    }

    pub(crate) fn validate_remove(
        &self,
        phys: PhysAddr,
    ) -> Result<MappedFrameRemove, PageTableError> {
        self.validate_len(self.len as u64)?;
        match self.search(phys.get())? {
            Ok(position) => Ok(MappedFrameRemove { position }),
            Err(_position) => Err(PageTableError::CorruptTable),
        }
    }

    pub(crate) fn remove_validated(
        &mut self,
        remove: MappedFrameRemove,
    ) -> Result<(), PageTableError> {
        if remove.position >= self.len {
            return Err(PageTableError::CorruptTable);
        }

        let mut index = remove.position;
        while index + 1 < self.len {
            self.set(index, self.get(index + 1)?)?;
            index += 1;
        }
        self.len -= 1;
        self.set(self.len, MappedFrameIndexEntry::EMPTY)?;
        Ok(())
    }

    pub(crate) fn validate(&self, expected_len: u64) -> Result<(), PageTableError> {
        self.validate_len(expected_len)?;

        let mut index = 1usize;
        while index < self.len {
            if self.get(index - 1)?.phys >= self.get(index)?.phys {
                return Err(PageTableError::CorruptTable);
            }
            index += 1;
        }
        Ok(())
    }

    pub(crate) fn position_of(&self, phys: PhysAddr) -> Result<usize, PageTableError> {
        match self.search(phys.get())? {
            Ok(position) => Ok(position),
            Err(_position) => Err(PageTableError::CorruptTable),
        }
    }

    pub(crate) fn validate_seen(&self, seen: &[bool; ENTRIES]) -> Result<(), PageTableError> {
        let mut index = 0usize;
        while index < self.len {
            if !seen[index] {
                return Err(PageTableError::CorruptTable);
            }
            index += 1;
        }
        Ok(())
    }

    fn validate_len(&self, expected_len: u64) -> Result<(), PageTableError> {
        if self.len > self.capacity()? || self.len as u64 != expected_len {
            return Err(PageTableError::CorruptTable);
        }
        Ok(())
    }

    fn search(&self, phys: u64) -> Result<Result<usize, usize>, PageTableError> {
        let mut low = 0usize;
        let mut high = self.len;
        while low < high {
            let mid = low + ((high - low) / 2);
            let current = self.get(mid)?.phys;
            if current == phys {
                return Ok(Ok(mid));
            }
            if current < phys {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        Ok(Err(low))
    }

    fn get(&self, index: usize) -> Result<MappedFrameIndexEntry, PageTableError> {
        if index >= ENTRIES {
            return Err(PageTableError::CorruptTable);
        }
        Ok(self.frames[index])
    }

    fn set(&mut self, index: usize, entry: MappedFrameIndexEntry) -> Result<(), PageTableError> {
        if index >= ENTRIES {
            return Err(PageTableError::CorruptTable);
        }
        self.frames[index] = entry;
        Ok(())
    }

    fn capacity(&self) -> Result<usize, PageTableError> {
        if ENTRIES == 0 {
            return Err(PageTableError::EmptyArena);
        }
        Ok(ENTRIES)
    }
}

impl MappedFrameIndexEntry {
    const EMPTY: Self = Self { phys: 0 };

    const fn new(phys: PhysAddr) -> Self {
        Self { phys: phys.get() }
    }
}
