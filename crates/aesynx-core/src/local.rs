use aesynx_abi::CoreId;

use crate::{CoreCapabilitySet, CoreError, CoreRole};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreState {
    Offline,
    Booting,
    Online,
    Quarantined,
}

impl CoreState {
    #[must_use]
    pub const fn is_live(self) -> bool {
        matches!(self, Self::Booting | Self::Online)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoreLocalTelemetry {
    role_assignments: u64,
    boot_barrier_arrivals: u64,
    local_events: u64,
}

impl CoreLocalTelemetry {
    pub fn record_role_assignment(&mut self) -> Result<(), CoreError> {
        self.role_assignments = self
            .role_assignments
            .checked_add(1)
            .ok_or(CoreError::TelemetryOverflow)?;
        Ok(())
    }

    pub fn record_boot_barrier_arrival(&mut self) -> Result<(), CoreError> {
        self.boot_barrier_arrivals = self
            .boot_barrier_arrivals
            .checked_add(1)
            .ok_or(CoreError::TelemetryOverflow)?;
        Ok(())
    }

    pub fn record_local_event(&mut self) -> Result<(), CoreError> {
        self.local_events = self
            .local_events
            .checked_add(1)
            .ok_or(CoreError::TelemetryOverflow)?;
        Ok(())
    }

    #[must_use]
    pub const fn role_assignments(self) -> u64 {
        self.role_assignments
    }

    #[must_use]
    pub const fn boot_barrier_arrivals(self) -> u64 {
        self.boot_barrier_arrivals
    }

    #[must_use]
    pub const fn local_events(self) -> u64 {
        self.local_events
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreLocal {
    id: CoreId,
    role: CoreRole,
    capabilities: CoreCapabilitySet,
    state: CoreState,
    telemetry: CoreLocalTelemetry,
}

impl CoreLocal {
    #[must_use]
    pub const fn new(
        id: CoreId,
        role: CoreRole,
        capabilities: CoreCapabilitySet,
        state: CoreState,
    ) -> Self {
        Self {
            id,
            role,
            capabilities,
            state,
            telemetry: CoreLocalTelemetry {
                role_assignments: 0,
                boot_barrier_arrivals: 0,
                local_events: 0,
            },
        }
    }

    #[must_use]
    pub const fn id(self) -> CoreId {
        self.id
    }

    #[must_use]
    pub const fn role(self) -> CoreRole {
        self.role
    }

    pub fn assign_role(&mut self, role: CoreRole) -> Result<(), CoreError> {
        self.telemetry.record_role_assignment()?;
        self.role = role;
        Ok(())
    }

    #[must_use]
    pub const fn capabilities(self) -> CoreCapabilitySet {
        self.capabilities
    }

    #[must_use]
    pub const fn state(self) -> CoreState {
        self.state
    }

    pub fn stage_startup(&mut self) -> Result<(), CoreError> {
        if self.state != CoreState::Offline {
            return Err(CoreError::InvalidStateTransition);
        }
        self.state = CoreState::Booting;
        Ok(())
    }

    pub fn mark_online(&mut self) -> Result<(), CoreError> {
        if self.state != CoreState::Booting {
            return Err(CoreError::InvalidStateTransition);
        }
        self.state = CoreState::Online;
        Ok(())
    }

    pub fn quarantine(&mut self) -> Result<(), CoreError> {
        if self.state == CoreState::Quarantined {
            return Err(CoreError::InvalidStateTransition);
        }
        self.state = CoreState::Quarantined;
        Ok(())
    }

    #[must_use]
    pub const fn telemetry(self) -> CoreLocalTelemetry {
        self.telemetry
    }

    pub fn telemetry_mut(&mut self) -> &mut CoreLocalTelemetry {
        &mut self.telemetry
    }

    #[must_use]
    pub const fn is_live(self) -> bool {
        self.state.is_live()
    }
}
