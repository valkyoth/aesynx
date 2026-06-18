use aesynx_abi::{CoreId, CpuHardwareId, VirtAddr};
use core::fmt;

use crate::{CoreError, CoreHardwareState, CoreState, CoreTopologyEntry};

pub const MIN_AP_STACK_BYTES: u64 = 16 * 1024;
const PAGE_SIZE: u64 = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApDescriptorTableReadiness {
    SharedBootstrapOnly,
    PerCoreReady,
}

impl ApDescriptorTableReadiness {
    #[must_use]
    pub const fn is_per_core_ready(self) -> bool {
        matches!(self, Self::PerCoreReady)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ApStartupResource {
    core: CoreId,
    hardware_id: CpuHardwareId,
    stack_base: VirtAddr,
    stack_len: u64,
    descriptor_tables: ApDescriptorTableReadiness,
    watchdog_ticks: u64,
}

impl fmt::Debug for ApStartupResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApStartupResource")
            .field("core", &self.core)
            .field("hardware_id", &"<redacted>")
            .field("stack_base", &"<redacted>")
            .field("stack_len", &self.stack_len)
            .field("descriptor_tables", &self.descriptor_tables)
            .field("watchdog_ticks", &self.watchdog_ticks)
            .finish()
    }
}

impl ApStartupResource {
    #[must_use]
    pub const fn core(self) -> CoreId {
        self.core
    }

    #[must_use]
    pub const fn hardware_id(self) -> CpuHardwareId {
        self.hardware_id
    }

    #[must_use]
    pub const fn stack_base(self) -> VirtAddr {
        self.stack_base
    }

    #[must_use]
    pub const fn stack_len(self) -> u64 {
        self.stack_len
    }

    #[must_use]
    pub const fn descriptor_tables(self) -> ApDescriptorTableReadiness {
        self.descriptor_tables
    }

    #[must_use]
    pub const fn watchdog_ticks(self) -> u64 {
        self.watchdog_ticks
    }

    fn stack_end(self) -> Result<u64, CoreError> {
        self.stack_base
            .get()
            .checked_add(self.stack_len)
            .ok_or(CoreError::InvalidStartupStack)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApStartupPreflightStatus {
    owner_core: CoreId,
    planned: usize,
    stack_ready: usize,
    descriptor_ready: usize,
    watchdog_ready: usize,
    capacity: usize,
}

impl ApStartupPreflightStatus {
    #[must_use]
    pub const fn owner_core(self) -> CoreId {
        self.owner_core
    }

    #[must_use]
    pub const fn planned(self) -> usize {
        self.planned
    }

    #[must_use]
    pub const fn stack_ready(self) -> usize {
        self.stack_ready
    }

    #[must_use]
    pub const fn descriptor_ready(self) -> usize {
        self.descriptor_ready
    }

    #[must_use]
    pub const fn watchdog_ready(self) -> usize {
        self.watchdog_ready
    }

    #[must_use]
    pub const fn capacity(self) -> usize {
        self.capacity
    }

    #[must_use]
    pub const fn execution_allowed(self) -> bool {
        self.planned != 0
            && self.stack_ready == self.planned
            && self.descriptor_ready == self.planned
            && self.watchdog_ready == self.planned
    }
}

#[derive(Eq, PartialEq)]
pub struct ApStartupPreflight<const CAPACITY: usize> {
    owner_core: CoreId,
    resources: [Option<ApStartupResource>; CAPACITY],
    len: usize,
}

impl<const CAPACITY: usize> ApStartupPreflight<CAPACITY> {
    pub const fn new(owner_core: CoreId) -> Result<Self, CoreError> {
        if CAPACITY == 0 {
            return Err(CoreError::CapacityZero);
        }

        Ok(Self {
            owner_core,
            resources: [const { None }; CAPACITY],
            len: 0,
        })
    }

    pub fn add_staged_core(
        &mut self,
        caller: CoreId,
        entry: CoreTopologyEntry,
        stack_base: VirtAddr,
        stack_len: u64,
        descriptor_tables: ApDescriptorTableReadiness,
        watchdog_ticks: u64,
    ) -> Result<(), CoreError> {
        self.require_owner(caller)?;
        if self.len == CAPACITY {
            return Err(CoreError::RegistryFull);
        }
        if entry.hardware_state() != CoreHardwareState::StartupStaged
            || entry.local_state() != CoreState::Booting
        {
            return Err(CoreError::InvalidStateTransition);
        }
        validate_stack(stack_base, stack_len)?;
        if watchdog_ticks == 0 {
            return Err(CoreError::MissingStartupWatchdog);
        }

        let resource = ApStartupResource {
            core: entry.core(),
            hardware_id: entry.hardware_id(),
            stack_base,
            stack_len,
            descriptor_tables,
            watchdog_ticks,
        };
        self.validate_unique_resource(resource)?;

        self.resources[self.len] = Some(resource);
        self.len += 1;
        Ok(())
    }

    #[must_use]
    pub fn status(&self) -> ApStartupPreflightStatus {
        let mut stack_ready = 0usize;
        let mut descriptor_ready = 0usize;
        let mut watchdog_ready = 0usize;
        let mut index = 0usize;

        while index < self.len {
            if let Some(resource) = self.resources[index] {
                if validate_stack(resource.stack_base, resource.stack_len).is_ok() {
                    stack_ready += 1;
                }
                if resource.descriptor_tables.is_per_core_ready() {
                    descriptor_ready += 1;
                }
                if resource.watchdog_ticks != 0 {
                    watchdog_ready += 1;
                }
            }
            index += 1;
        }

        ApStartupPreflightStatus {
            owner_core: self.owner_core,
            planned: self.len,
            stack_ready,
            descriptor_ready,
            watchdog_ready,
            capacity: CAPACITY,
        }
    }

    #[must_use]
    pub fn resource(&self, core: CoreId) -> Option<ApStartupResource> {
        let mut index = 0usize;
        while index < self.len {
            if let Some(resource) = self.resources[index]
                && resource.core == core
            {
                return Some(resource);
            }
            index += 1;
        }
        None
    }

    fn require_owner(&self, caller: CoreId) -> Result<(), CoreError> {
        if caller != self.owner_core {
            return Err(CoreError::OwnerMismatch);
        }
        Ok(())
    }

    fn validate_unique_resource(&self, resource: ApStartupResource) -> Result<(), CoreError> {
        let end = resource.stack_end()?;
        let mut index = 0usize;
        while index < self.len {
            if let Some(existing) = self.resources[index] {
                if existing.core == resource.core {
                    return Err(CoreError::DuplicateCore);
                }
                if existing.hardware_id == resource.hardware_id {
                    return Err(CoreError::DuplicateHardwareId);
                }

                let existing_end = existing.stack_end()?;
                if resource.stack_base.get() < existing_end && existing.stack_base.get() < end {
                    return Err(CoreError::DuplicateStartupStack);
                }
            }
            index += 1;
        }
        Ok(())
    }
}

fn validate_stack(base: VirtAddr, len: u64) -> Result<(), CoreError> {
    if len < MIN_AP_STACK_BYTES {
        return Err(CoreError::InvalidStartupStack);
    }
    if !is_page_aligned(base.get()) || !is_page_aligned(len) {
        return Err(CoreError::InvalidStartupStack);
    }
    base.get()
        .checked_add(len)
        .ok_or(CoreError::InvalidStartupStack)?;
    Ok(())
}

const fn is_page_aligned(value: u64) -> bool {
    value & (PAGE_SIZE - 1) == 0
}
