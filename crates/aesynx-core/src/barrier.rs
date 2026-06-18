use aesynx_abi::CoreId;

use crate::CoreError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootBarrierStatus {
    owner_core: CoreId,
    participants: usize,
    arrivals: usize,
    sealed: bool,
}

impl BootBarrierStatus {
    #[must_use]
    pub const fn owner_core(self) -> CoreId {
        self.owner_core
    }

    #[must_use]
    pub const fn participants(self) -> usize {
        self.participants
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.participants == 0
    }

    #[must_use]
    pub const fn arrivals(self) -> usize {
        self.arrivals
    }

    #[must_use]
    pub const fn sealed(self) -> bool {
        self.sealed
    }

    #[must_use]
    pub const fn all_arrived(self) -> bool {
        self.sealed && self.participants == self.arrivals
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct BootBarrier<const CAPACITY: usize> {
    owner_core: CoreId,
    participants: [Option<CoreId>; CAPACITY],
    arrived: [bool; CAPACITY],
    len: usize,
    arrivals: usize,
    sealed: bool,
}

impl<const CAPACITY: usize> BootBarrier<CAPACITY> {
    pub const fn new(owner_core: CoreId) -> Result<Self, CoreError> {
        if CAPACITY == 0 {
            return Err(CoreError::CapacityZero);
        }

        Ok(Self {
            owner_core,
            participants: [const { None }; CAPACITY],
            arrived: [false; CAPACITY],
            len: 0,
            arrivals: 0,
            sealed: false,
        })
    }

    pub fn add_participant(&mut self, caller: CoreId, core: CoreId) -> Result<(), CoreError> {
        self.require_owner(caller)?;
        if self.sealed {
            return Err(CoreError::BarrierSealed);
        }
        if self.len == CAPACITY {
            return Err(CoreError::RegistryFull);
        }
        if self.index_of(core).is_some() {
            return Err(CoreError::DuplicateCore);
        }

        self.participants[self.len] = Some(core);
        self.len += 1;
        Ok(())
    }

    pub fn seal(&mut self, caller: CoreId) -> Result<(), CoreError> {
        self.require_owner(caller)?;
        if self.sealed {
            return Err(CoreError::BarrierSealed);
        }
        if self.len == 0 {
            return Err(CoreError::UnknownCore);
        }
        self.sealed = true;
        Ok(())
    }

    fn require_owner(&self, caller: CoreId) -> Result<(), CoreError> {
        if caller != self.owner_core {
            return Err(CoreError::OwnerMismatch);
        }
        Ok(())
    }

    pub fn arrive(&mut self, core: CoreId) -> Result<(), CoreError> {
        if !self.sealed {
            return Err(CoreError::BarrierNotSealed);
        }
        let Some(index) = self.index_of(core) else {
            return Err(CoreError::UnknownCore);
        };
        if self.arrived[index] {
            return Err(CoreError::AlreadyArrived);
        }

        self.arrived[index] = true;
        self.arrivals = self
            .arrivals
            .checked_add(1)
            .ok_or(CoreError::TelemetryOverflow)?;
        Ok(())
    }

    #[must_use]
    pub const fn status(&self) -> BootBarrierStatus {
        BootBarrierStatus {
            owner_core: self.owner_core,
            participants: self.len,
            arrivals: self.arrivals,
            sealed: self.sealed,
        }
    }

    #[must_use]
    fn index_of(&self, core: CoreId) -> Option<usize> {
        let mut index = 0usize;
        while index < self.len {
            if self.participants[index] == Some(core) {
                return Some(index);
            }
            index += 1;
        }
        None
    }
}
