use core::fmt;

use aesynx_abi::{CoreId, CpuHardwareId};

use crate::{
    CoreCapabilitySet, CoreError, CoreLocal, CoreLocalTelemetry, CoreRole, CoreStartupArrival,
    CoreStartupJointState, CoreStartupTicket, CoreState,
};

pub const QEMU_MULTICORE_TOPOLOGY_CORES: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreHardwareState {
    Discovered,
    StartupStaged,
    Online,
    Quarantined,
}

impl CoreHardwareState {
    #[must_use]
    pub const fn is_online(self) -> bool {
        matches!(self, Self::Online)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreAssignmentState {
    Unassigned,
    Assigned,
}

impl CoreAssignmentState {
    #[must_use]
    pub const fn is_assigned(self) -> bool {
        matches!(self, Self::Assigned)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CoreTopologyEntry {
    hardware_id: CpuHardwareId,
    local: CoreLocal,
    hardware_state: CoreHardwareState,
    assignment_state: CoreAssignmentState,
    staged_epoch: Option<u64>,
}

impl CoreTopologyEntry {
    #[must_use]
    pub const fn discovered(
        core: CoreId,
        hardware_id: CpuHardwareId,
        capabilities: CoreCapabilitySet,
    ) -> Self {
        Self {
            hardware_id,
            local: CoreLocal::new(core, CoreRole::Idle, capabilities, CoreState::Offline),
            hardware_state: CoreHardwareState::Discovered,
            assignment_state: CoreAssignmentState::Unassigned,
            staged_epoch: None,
        }
    }

    #[must_use]
    pub const fn core(self) -> CoreId {
        self.local.id()
    }

    #[must_use]
    pub const fn hardware_id(self) -> CpuHardwareId {
        self.hardware_id
    }

    #[must_use]
    pub const fn role(self) -> CoreRole {
        self.local.role()
    }

    #[must_use]
    pub const fn capabilities(self) -> CoreCapabilitySet {
        self.local.capabilities()
    }

    #[must_use]
    pub const fn hardware_state(self) -> CoreHardwareState {
        self.hardware_state
    }

    #[must_use]
    pub const fn assignment_state(self) -> CoreAssignmentState {
        self.assignment_state
    }

    #[must_use]
    pub const fn local_state(self) -> CoreState {
        self.local.state()
    }

    #[must_use]
    pub const fn startup_state(self) -> CoreStartupJointState {
        CoreStartupJointState::new(
            self.hardware_state,
            self.assignment_state,
            self.local.state(),
        )
    }

    #[must_use]
    pub const fn telemetry(self) -> CoreLocalTelemetry {
        self.local.telemetry()
    }

    fn validate_startup_state(self) -> Result<(), CoreError> {
        self.startup_state().validate()
    }
}

impl fmt::Debug for CoreTopologyEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CoreTopologyEntry")
            .field("hardware_id", &"<redacted>")
            .field("local", &self.local)
            .field("hardware_state", &self.hardware_state)
            .field("assignment_state", &self.assignment_state)
            .field("staged_epoch", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreTopologyStatus {
    owner_core: CoreId,
    discovered: usize,
    hardware_online: usize,
    assigned: usize,
    bootstrap_roles: usize,
    scheduler_roles: usize,
    driver_service_roles: usize,
    idle_roles: usize,
    capacity: usize,
}

impl CoreTopologyStatus {
    #[must_use]
    pub const fn owner_core(self) -> CoreId {
        self.owner_core
    }

    #[must_use]
    pub const fn discovered(self) -> usize {
        self.discovered
    }

    #[must_use]
    pub const fn hardware_online(self) -> usize {
        self.hardware_online
    }

    #[must_use]
    pub const fn assigned(self) -> usize {
        self.assigned
    }

    #[must_use]
    pub const fn bootstrap_roles(self) -> usize {
        self.bootstrap_roles
    }

    #[must_use]
    pub const fn scheduler_roles(self) -> usize {
        self.scheduler_roles
    }

    #[must_use]
    pub const fn driver_service_roles(self) -> usize {
        self.driver_service_roles
    }

    #[must_use]
    pub const fn idle_roles(self) -> usize {
        self.idle_roles
    }

    #[must_use]
    pub const fn capacity(self) -> usize {
        self.capacity
    }
}

#[derive(Eq, PartialEq)]
pub struct CoreTopology<const CAPACITY: usize> {
    owner_core: CoreId,
    entries: [Option<CoreTopologyEntry>; CAPACITY],
    len: usize,
    epoch: u64,
}

impl<const CAPACITY: usize> fmt::Debug for CoreTopology<CAPACITY> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CoreTopology")
            .field("owner_core", &self.owner_core)
            .field("len", &self.len)
            .field("capacity", &CAPACITY)
            .field("epoch", &"<redacted>")
            .finish()
    }
}

impl<const CAPACITY: usize> CoreTopology<CAPACITY> {
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

    pub fn insert_discovered(
        &mut self,
        caller: CoreId,
        core: CoreId,
        hardware_id: CpuHardwareId,
        capabilities: CoreCapabilitySet,
    ) -> Result<(), CoreError> {
        self.require_owner(caller)?;
        if self.len == CAPACITY {
            return Err(CoreError::RegistryFull);
        }
        if self.index_of_core(core).is_some() {
            return Err(CoreError::DuplicateCore);
        }
        if self.index_of_hardware_id(hardware_id).is_some() {
            return Err(CoreError::DuplicateHardwareId);
        }
        self.bump_epoch()?;

        self.entries[self.len] = Some(CoreTopologyEntry::discovered(
            core,
            hardware_id,
            capabilities,
        ));
        self.len += 1;
        Ok(())
    }

    pub fn stage_startup_ticket(
        &mut self,
        caller: CoreId,
        core: CoreId,
    ) -> Result<CoreStartupTicket, CoreError> {
        self.require_owner(caller)?;
        let index = self.index_of_core(core).ok_or(CoreError::UnknownCore)?;
        let Some(mut entry) = self.entries[index] else {
            return Err(CoreError::UnknownCore);
        };
        entry.startup_state().validate_startup_stage()?;

        entry.hardware_state = CoreHardwareState::StartupStaged;
        entry.local.stage_startup()?;
        entry.local.telemetry_mut().record_local_event()?;
        let next_epoch = self.next_epoch()?;
        entry.staged_epoch = Some(next_epoch);
        entry.validate_startup_state()?;
        let ticket = CoreStartupTicket::new(core, entry.hardware_id, caller, next_epoch);
        self.epoch = next_epoch;
        self.entries[index] = Some(entry);
        Ok(ticket)
    }

    pub fn mark_hardware_online(
        &mut self,
        caller: CoreId,
        arrival: CoreStartupArrival,
    ) -> Result<(), CoreError> {
        self.require_owner(caller)?;
        if arrival.coordinator_core() != self.owner_core {
            return Err(CoreError::OwnerMismatch);
        }
        let core = arrival.arrived_core();
        let index = self.index_of_core(core).ok_or(CoreError::UnknownCore)?;
        let Some(mut entry) = self.entries[index] else {
            return Err(CoreError::UnknownCore);
        };
        entry.startup_state().validate_hardware_online()?;
        if entry.staged_epoch != Some(arrival.startup_epoch()) {
            return Err(CoreError::InvalidStartupEpoch);
        }
        if entry.hardware_id != arrival.hardware_id() {
            return Err(CoreError::StartupEvidenceMismatch);
        }

        entry.hardware_state = CoreHardwareState::Online;
        entry.local.mark_online()?;
        entry.local.telemetry_mut().record_local_event()?;
        entry.validate_startup_state()?;
        self.bump_epoch()?;
        self.entries[index] = Some(entry);
        Ok(())
    }

    pub fn assign_role(
        &mut self,
        caller: CoreId,
        core: CoreId,
        role: CoreRole,
    ) -> Result<(), CoreError> {
        self.require_owner(caller)?;
        let index = self.index_of_core(core).ok_or(CoreError::UnknownCore)?;
        let Some(mut entry) = self.entries[index] else {
            return Err(CoreError::UnknownCore);
        };
        entry.startup_state().validate_role_assignment()?;

        entry.local.assign_role(role)?;
        entry.assignment_state = CoreAssignmentState::Assigned;
        entry.validate_startup_state()?;
        self.bump_epoch()?;
        self.entries[index] = Some(entry);
        Ok(())
    }

    pub fn quarantine(&mut self, caller: CoreId, core: CoreId) -> Result<(), CoreError> {
        self.require_owner(caller)?;
        let index = self.index_of_core(core).ok_or(CoreError::UnknownCore)?;
        let Some(mut entry) = self.entries[index] else {
            return Err(CoreError::UnknownCore);
        };
        entry.startup_state().validate_quarantine()?;

        entry.hardware_state = CoreHardwareState::Quarantined;
        entry.local.quarantine()?;
        entry.local.telemetry_mut().record_local_event()?;
        entry.validate_startup_state()?;
        self.bump_epoch()?;
        self.entries[index] = Some(entry);
        Ok(())
    }

    #[must_use]
    pub fn get(&self, core: CoreId) -> Option<CoreTopologyEntry> {
        self.index_of_core(core)
            .and_then(|index| self.entries[index])
    }

    #[must_use]
    pub fn status(&self) -> CoreTopologyStatus {
        let mut hardware_online = 0usize;
        let mut assigned = 0usize;
        let mut bootstrap_roles = 0usize;
        let mut scheduler_roles = 0usize;
        let mut driver_service_roles = 0usize;
        let mut idle_roles = 0usize;
        let mut index = 0usize;

        while index < self.len {
            if let Some(entry) = self.entries[index] {
                if entry.hardware_state.is_online() {
                    hardware_online += 1;
                }
                if entry.hardware_state != CoreHardwareState::Quarantined
                    && entry.assignment_state.is_assigned()
                {
                    assigned += 1;
                }
                match (entry.hardware_state, entry.role(), entry.assignment_state) {
                    (CoreHardwareState::Quarantined, _, _) => {}
                    (_, CoreRole::Idle, CoreAssignmentState::Unassigned) => {}
                    (_, CoreRole::Idle, CoreAssignmentState::Assigned) => idle_roles += 1,
                    (_, CoreRole::Bootstrap, _) => bootstrap_roles += 1,
                    (_, CoreRole::Scheduler, _) => scheduler_roles += 1,
                    (_, CoreRole::DriverService, _) => driver_service_roles += 1,
                }
            }
            index += 1;
        }

        CoreTopologyStatus {
            owner_core: self.owner_core,
            discovered: self.len,
            hardware_online,
            assigned,
            bootstrap_roles,
            scheduler_roles,
            driver_service_roles,
            idle_roles,
            capacity: CAPACITY,
        }
    }

    fn require_owner(&self, caller: CoreId) -> Result<(), CoreError> {
        if caller != self.owner_core {
            return Err(CoreError::OwnerMismatch);
        }
        Ok(())
    }

    fn bump_epoch(&mut self) -> Result<(), CoreError> {
        self.epoch = self.next_epoch()?;
        Ok(())
    }

    fn next_epoch(&self) -> Result<u64, CoreError> {
        self.epoch
            .checked_add(1)
            .ok_or(CoreError::TelemetryOverflow)
    }

    fn index_of_core(&self, core: CoreId) -> Option<usize> {
        let mut index = 0usize;
        while index < self.len {
            if self.entries[index].is_some_and(|entry| entry.core() == core) {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    fn index_of_hardware_id(&self, hardware_id: CpuHardwareId) -> Option<usize> {
        let mut index = 0usize;
        while index < self.len {
            if self.entries[index].is_some_and(|entry| entry.hardware_id() == hardware_id) {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    #[cfg(test)]
    pub(crate) const fn with_epoch_for_test(mut self, epoch: u64) -> Self {
        self.epoch = epoch;
        self
    }
}
