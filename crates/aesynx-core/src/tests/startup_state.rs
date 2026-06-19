use crate::{
    CoreAssignmentState, CoreError, CoreHardwareState, CoreStartupJointState, CoreState,
    audit_startup_state_table,
};

#[test]
fn startup_state_table_accepts_only_jointly_valid_combinations() {
    assert!(
        CoreStartupJointState::new(
            CoreHardwareState::Discovered,
            CoreAssignmentState::Unassigned,
            CoreState::Offline,
        )
        .is_valid()
    );
    assert!(
        CoreStartupJointState::new(
            CoreHardwareState::StartupStaged,
            CoreAssignmentState::Assigned,
            CoreState::Booting,
        )
        .is_valid()
    );
    assert!(
        CoreStartupJointState::new(
            CoreHardwareState::Online,
            CoreAssignmentState::Assigned,
            CoreState::Online,
        )
        .is_valid()
    );
    assert!(
        CoreStartupJointState::new(
            CoreHardwareState::Quarantined,
            CoreAssignmentState::Assigned,
            CoreState::Quarantined,
        )
        .is_valid()
    );

    assert!(
        !CoreStartupJointState::new(
            CoreHardwareState::Discovered,
            CoreAssignmentState::Assigned,
            CoreState::Online,
        )
        .is_valid()
    );
    assert!(
        !CoreStartupJointState::new(
            CoreHardwareState::StartupStaged,
            CoreAssignmentState::Assigned,
            CoreState::Offline,
        )
        .is_valid()
    );
    assert!(
        !CoreStartupJointState::new(
            CoreHardwareState::Online,
            CoreAssignmentState::Assigned,
            CoreState::Booting,
        )
        .is_valid()
    );
    assert!(
        !CoreStartupJointState::new(
            CoreHardwareState::Quarantined,
            CoreAssignmentState::Assigned,
            CoreState::Online,
        )
        .is_valid()
    );
}

#[test]
fn startup_state_table_counts_every_axis_combination() {
    let status = audit_startup_state_table();

    assert_eq!(status.valid_combinations(), 8);
    assert_eq!(status.invalid_combinations(), 24);
    assert_eq!(status.total_combinations(), 32);
}

#[test]
fn startup_state_table_rejects_invalid_transition_intents() {
    let discovered = CoreStartupJointState::new(
        CoreHardwareState::Discovered,
        CoreAssignmentState::Unassigned,
        CoreState::Offline,
    );
    let staged = CoreStartupJointState::new(
        CoreHardwareState::StartupStaged,
        CoreAssignmentState::Assigned,
        CoreState::Booting,
    );
    let online = CoreStartupJointState::new(
        CoreHardwareState::Online,
        CoreAssignmentState::Assigned,
        CoreState::Online,
    );
    let quarantined = CoreStartupJointState::new(
        CoreHardwareState::Quarantined,
        CoreAssignmentState::Assigned,
        CoreState::Quarantined,
    );

    assert!(discovered.validate_startup_stage().is_ok());
    assert!(discovered.validate_role_assignment().is_ok());
    assert!(discovered.validate_quarantine().is_ok());
    assert_eq!(
        discovered.validate_hardware_online().err(),
        Some(CoreError::InvalidStateTransition)
    );

    assert!(staged.validate_hardware_online().is_ok());
    assert!(staged.validate_role_assignment().is_ok());
    assert!(staged.validate_quarantine().is_ok());
    assert_eq!(
        staged.validate_startup_stage().err(),
        Some(CoreError::InvalidStateTransition)
    );

    assert_eq!(
        online.validate_role_assignment().err(),
        Some(CoreError::InvalidStateTransition)
    );
    assert_eq!(
        online.validate_startup_stage().err(),
        Some(CoreError::InvalidStateTransition)
    );
    assert_eq!(
        online.validate_quarantine().err(),
        Some(CoreError::InvalidStateTransition)
    );

    assert_eq!(
        quarantined.validate_quarantine().err(),
        Some(CoreError::InvalidStateTransition)
    );
    assert_eq!(
        quarantined.validate_role_assignment().err(),
        Some(CoreError::InvalidStateTransition)
    );
}
