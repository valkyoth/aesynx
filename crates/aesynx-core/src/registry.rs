use aesynx_abi::CoreId;

use crate::{CoreError, CoreLocal, CoreRole};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreRegistryStatus {
    owner_core: CoreId,
    len: usize,
    capacity: usize,
    epoch: u64,
}

impl CoreRegistryStatus {
    #[must_use]
    pub const fn owner_core(self) -> CoreId {
        self.owner_core
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    #[must_use]
    pub const fn capacity(self) -> usize {
        self.capacity
    }

    #[must_use]
    pub const fn epoch(self) -> u64 {
        self.epoch
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CoreRegistry<const CAPACITY: usize> {
    owner_core: CoreId,
    entries: [Option<CoreLocal>; CAPACITY],
    len: usize,
    epoch: u64,
}

impl<const CAPACITY: usize> CoreRegistry<CAPACITY> {
    pub const fn new(owner_core: CoreId) -> Result<Self, CoreError> {
        if CAPACITY == 0 {
            return Err(CoreError::CapacityZero);
        }

        Ok(Self {
            owner_core,
            entries: [const { None }; CAPACITY],
            len: 0,
            epoch: 0,
        })
    }

    #[must_use]
    pub const fn owner_core(&self) -> CoreId {
        self.owner_core
    }

    pub fn insert(&mut self, caller: CoreId, local: CoreLocal) -> Result<(), CoreError> {
        self.require_owner(caller)?;
        if self.len == CAPACITY {
            return Err(CoreError::RegistryFull);
        }
        if self.contains(local.id()) {
            return Err(CoreError::DuplicateCore);
        }
        let next_epoch = self
            .epoch
            .checked_add(1)
            .ok_or(CoreError::TelemetryOverflow)?;

        self.entries[self.len] = Some(local);
        self.len += 1;
        self.epoch = next_epoch;
        Ok(())
    }

    fn require_owner(&self, caller: CoreId) -> Result<(), CoreError> {
        if caller != self.owner_core {
            return Err(CoreError::OwnerMismatch);
        }
        Ok(())
    }

    #[must_use]
    pub fn contains(&self, core: CoreId) -> bool {
        self.get(core).is_some()
    }

    #[must_use]
    pub fn get(&self, core: CoreId) -> Option<CoreLocal> {
        let mut index = 0usize;
        while index < self.len {
            if let Some(local) = self.entries[index]
                && local.id() == core
            {
                return Some(local);
            }
            index += 1;
        }
        None
    }

    pub fn require_role(&self, core: CoreId, role: CoreRole) -> Result<CoreLocal, CoreError> {
        let Some(local) = self.get(core) else {
            return Err(CoreError::UnknownCore);
        };
        if local.role() != role {
            return Err(CoreError::RoleMismatch);
        }
        Ok(local)
    }

    #[must_use]
    pub fn live_count(&self) -> usize {
        let mut count = 0usize;
        let mut index = 0usize;
        while index < self.len {
            if self.entries[index].is_some_and(CoreLocal::is_live) {
                count += 1;
            }
            index += 1;
        }
        count
    }

    #[must_use]
    pub const fn status(&self) -> CoreRegistryStatus {
        CoreRegistryStatus {
            owner_core: self.owner_core,
            len: self.len,
            capacity: CAPACITY,
            epoch: self.epoch,
        }
    }

    #[cfg(test)]
    pub(crate) const fn with_epoch_for_test(mut self, epoch: u64) -> Self {
        self.epoch = epoch;
        self
    }
}
