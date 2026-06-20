use aesynx_abi::{CoreId, CpuHardwareId, VirtAddr};
use core::cell::Cell;
use core::fmt;
use core::marker::PhantomData;

use crate::{CoreError, CoreHardwareState, CoreState, CoreTopologyEntry};

pub const MIN_AP_STACK_BYTES: u64 = 32 * 1024;
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
pub struct ApStackRegion {
    start: VirtAddr,
    end: VirtAddr,
}

impl fmt::Debug for ApStackRegion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApStackRegion")
            .field("start", &"<redacted>")
            .field("end", &"<redacted>")
            .finish()
    }
}

impl ApStackRegion {
    pub const fn new(start: VirtAddr, end: VirtAddr) -> Result<Self, CoreError> {
        if start.get() >= end.get() || !is_page_aligned(start.get()) || !is_page_aligned(end.get())
        {
            return Err(CoreError::InvalidStartupStack);
        }

        Ok(Self { start, end })
    }

    #[cfg(test)]
    pub(crate) const fn test_only(start: VirtAddr, end: VirtAddr) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn start(self) -> VirtAddr {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> VirtAddr {
        self.end
    }

    const fn contains_stack(self, stack_base: VirtAddr, stack_end: u64) -> bool {
        stack_base.get() >= self.start.get() && stack_end <= self.end.get()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ApStackPlan {
    base: VirtAddr,
    len: u64,
    region: ApStackRegion,
}

impl fmt::Debug for ApStackPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApStackPlan")
            .field("base", &"<redacted>")
            .field("len", &self.len)
            .field("region", &"<redacted>")
            .finish()
    }
}

impl ApStackPlan {
    pub const fn new(base: VirtAddr, len: u64, region: ApStackRegion) -> Result<Self, CoreError> {
        if validate_stack(base, len, region).is_err() {
            return Err(CoreError::InvalidStartupStack);
        }

        Ok(Self { base, len, region })
    }

    #[must_use]
    pub const fn base(self) -> VirtAddr {
        self.base
    }

    #[must_use]
    pub const fn byte_len(self) -> u64 {
        self.len
    }

    #[must_use]
    pub const fn region(self) -> ApStackRegion {
        self.region
    }

    fn end(self) -> Result<u64, CoreError> {
        self.base
            .get()
            .checked_add(self.len)
            .ok_or(CoreError::InvalidStartupStack)
    }

    #[cfg(test)]
    pub(crate) const fn test_only(base: VirtAddr, len: u64, region: ApStackRegion) -> Self {
        Self { base, len, region }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ApStartupResource {
    core: CoreId,
    hardware_id: CpuHardwareId,
    stack: ApStackPlan,
    descriptor_tables: ApDescriptorTableReadiness,
    watchdog_ticks: u64,
}

impl fmt::Debug for ApStartupResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApStartupResource")
            .field("core", &self.core)
            .field("hardware_id", &"<redacted>")
            .field("stack", &"<redacted>")
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
        self.stack.base()
    }

    #[must_use]
    pub const fn stack_len(self) -> u64 {
        self.stack.byte_len()
    }

    #[must_use]
    pub const fn stack_region(self) -> ApStackRegion {
        self.stack.region()
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
        self.stack.end()
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

pub struct ApStartupDispatchToken<const CAPACITY: usize> {
    owner_core: CoreId,
    resources: [Option<ApStartupResource>; CAPACITY],
    len: usize,
    _not_sync: PhantomData<Cell<()>>,
}

impl<const CAPACITY: usize> fmt::Debug for ApStartupDispatchToken<CAPACITY> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApStartupDispatchToken")
            .field("owner_core", &self.owner_core)
            .field("planned", &self.len)
            .field("resources", &"<redacted>")
            .finish()
    }
}

impl<const CAPACITY: usize> Drop for ApStartupDispatchToken<CAPACITY> {
    fn drop(&mut self) {
        let mut index = 0usize;
        while index < CAPACITY {
            self.resources[index] = None;
            index += 1;
        }
        self.len = 0;
    }
}

impl<const CAPACITY: usize> ApStartupDispatchToken<CAPACITY> {
    #[must_use]
    pub const fn owner_core(&self) -> CoreId {
        self.owner_core
    }

    #[must_use]
    pub const fn planned(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        CAPACITY
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
        stack: ApStackPlan,
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
        if watchdog_ticks == 0 {
            return Err(CoreError::MissingStartupWatchdog);
        }

        let resource = ApStartupResource {
            core: entry.core(),
            hardware_id: entry.hardware_id(),
            stack,
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
                if validate_stack(
                    resource.stack_base(),
                    resource.stack_len(),
                    resource.stack_region(),
                )
                .is_ok()
                {
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

    pub fn into_dispatch_token(
        self,
        caller: CoreId,
    ) -> Result<ApStartupDispatchToken<CAPACITY>, CoreError> {
        self.require_owner(caller)?;
        if !self.status().execution_allowed() {
            return Err(CoreError::StartupPreflightBlocked);
        }

        Ok(ApStartupDispatchToken {
            owner_core: self.owner_core,
            resources: self.resources,
            len: self.len,
            _not_sync: PhantomData,
        })
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
                if resource.stack_base().get() < existing_end && existing.stack_base().get() < end {
                    return Err(CoreError::DuplicateStartupStack);
                }
            }
            index += 1;
        }
        Ok(())
    }
}

const fn validate_stack(base: VirtAddr, len: u64, region: ApStackRegion) -> Result<(), CoreError> {
    let base_raw = base.get();
    if len < MIN_AP_STACK_BYTES {
        return Err(CoreError::InvalidStartupStack);
    }
    if !is_page_aligned(base_raw) || !is_page_aligned(len) {
        return Err(CoreError::InvalidStartupStack);
    }
    let Some(end) = base_raw.checked_add(len) else {
        return Err(CoreError::InvalidStartupStack);
    };
    if !region.contains_stack(base, end) {
        return Err(CoreError::InvalidStartupStack);
    }
    Ok(())
}

const fn is_page_aligned(value: u64) -> bool {
    value & (PAGE_SIZE - 1) == 0
}
