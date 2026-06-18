use crate::{CoreAssignmentState, CoreError, CoreHardwareState, CoreState};

const HARDWARE_STATES: [CoreHardwareState; 4] = [
    CoreHardwareState::Discovered,
    CoreHardwareState::StartupStaged,
    CoreHardwareState::Online,
    CoreHardwareState::Quarantined,
];
const ASSIGNMENT_STATES: [CoreAssignmentState; 2] = [
    CoreAssignmentState::Unassigned,
    CoreAssignmentState::Assigned,
];
const LOCAL_STATES: [CoreState; 4] = [
    CoreState::Offline,
    CoreState::Booting,
    CoreState::Online,
    CoreState::Quarantined,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreStartupJointState {
    hardware_state: CoreHardwareState,
    assignment_state: CoreAssignmentState,
    local_state: CoreState,
}

impl CoreStartupJointState {
    #[must_use]
    pub const fn new(
        hardware_state: CoreHardwareState,
        assignment_state: CoreAssignmentState,
        local_state: CoreState,
    ) -> Self {
        Self {
            hardware_state,
            assignment_state,
            local_state,
        }
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
        self.local_state
    }

    #[must_use]
    pub const fn is_valid(self) -> bool {
        matches!(
            (self.hardware_state, self.assignment_state, self.local_state),
            (
                CoreHardwareState::Discovered,
                CoreAssignmentState::Unassigned | CoreAssignmentState::Assigned,
                CoreState::Offline
            ) | (
                CoreHardwareState::StartupStaged,
                CoreAssignmentState::Unassigned | CoreAssignmentState::Assigned,
                CoreState::Booting
            ) | (
                CoreHardwareState::Online,
                CoreAssignmentState::Unassigned | CoreAssignmentState::Assigned,
                CoreState::Online
            ) | (
                CoreHardwareState::Quarantined,
                CoreAssignmentState::Unassigned | CoreAssignmentState::Assigned,
                CoreState::Quarantined
            )
        )
    }

    pub fn validate(self) -> Result<(), CoreError> {
        if self.is_valid() {
            Ok(())
        } else {
            Err(CoreError::InvalidStateTransition)
        }
    }

    pub fn validate_startup_stage(self) -> Result<(), CoreError> {
        self.validate()?;
        if matches!(
            (self.hardware_state, self.local_state),
            (CoreHardwareState::Discovered, CoreState::Offline)
        ) {
            Ok(())
        } else {
            Err(CoreError::InvalidStateTransition)
        }
    }

    pub fn validate_role_assignment(self) -> Result<(), CoreError> {
        self.validate()?;
        match (self.hardware_state, self.local_state) {
            (CoreHardwareState::Discovered, CoreState::Offline)
            | (CoreHardwareState::StartupStaged, CoreState::Booting) => Ok(()),
            _ => Err(CoreError::InvalidStateTransition),
        }
    }

    pub fn validate_hardware_online(self) -> Result<(), CoreError> {
        self.validate()?;
        if matches!(
            (self.hardware_state, self.local_state),
            (CoreHardwareState::StartupStaged, CoreState::Booting)
        ) {
            Ok(())
        } else {
            Err(CoreError::InvalidStateTransition)
        }
    }

    pub fn validate_quarantine(self) -> Result<(), CoreError> {
        self.validate()?;
        if matches!(
            (self.hardware_state, self.local_state),
            (CoreHardwareState::Quarantined, CoreState::Quarantined)
        ) {
            Err(CoreError::InvalidStateTransition)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreStartupStateTableStatus {
    valid_combinations: usize,
    invalid_combinations: usize,
}

impl CoreStartupStateTableStatus {
    #[must_use]
    pub const fn valid_combinations(self) -> usize {
        self.valid_combinations
    }

    #[must_use]
    pub const fn invalid_combinations(self) -> usize {
        self.invalid_combinations
    }

    #[must_use]
    pub const fn total_combinations(self) -> usize {
        self.valid_combinations + self.invalid_combinations
    }
}

#[must_use]
pub fn audit_startup_state_table() -> CoreStartupStateTableStatus {
    let mut valid_combinations = 0usize;
    let mut invalid_combinations = 0usize;
    let mut hardware_index = 0usize;

    while hardware_index < HARDWARE_STATES.len() {
        let mut assignment_index = 0usize;
        while assignment_index < ASSIGNMENT_STATES.len() {
            let mut local_index = 0usize;
            while local_index < LOCAL_STATES.len() {
                let state = CoreStartupJointState::new(
                    HARDWARE_STATES[hardware_index],
                    ASSIGNMENT_STATES[assignment_index],
                    LOCAL_STATES[local_index],
                );
                if state.is_valid() {
                    valid_combinations += 1;
                } else {
                    invalid_combinations += 1;
                }
                local_index += 1;
            }
            assignment_index += 1;
        }
        hardware_index += 1;
    }

    CoreStartupStateTableStatus {
        valid_combinations,
        invalid_combinations,
    }
}
