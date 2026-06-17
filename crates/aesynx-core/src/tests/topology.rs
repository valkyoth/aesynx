use aesynx_abi::{CoreId, CpuHardwareId, ROOT_CORE};

use crate::{CoreError, CoreHardwareState, CoreRole, CoreState, CoreTopology};

use super::qemu_bootstrap_caps;

#[test]
fn core_topology_rejects_online_role_reassignment() {
    let mut topology = match CoreTopology::<1>::new(ROOT_CORE) {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };
    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                ROOT_CORE,
                CpuHardwareId::new(0),
                qemu_bootstrap_caps()
            )
            .is_ok()
    );
    assert!(topology.stage_startup(ROOT_CORE, ROOT_CORE).is_ok());
    assert!(
        topology
            .assign_role(ROOT_CORE, ROOT_CORE, CoreRole::Bootstrap)
            .is_ok()
    );
    assert!(topology.mark_hardware_online(ROOT_CORE, ROOT_CORE).is_ok());
    let before = topology.get(ROOT_CORE);

    assert_eq!(
        topology
            .assign_role(ROOT_CORE, ROOT_CORE, CoreRole::Idle)
            .err(),
        Some(CoreError::InvalidStateTransition)
    );
    assert_eq!(topology.get(ROOT_CORE), before);
}

#[test]
fn core_topology_rejects_online_without_startup_staging() {
    let mut topology = match CoreTopology::<1>::new(ROOT_CORE) {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };
    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                ROOT_CORE,
                CpuHardwareId::new(0),
                qemu_bootstrap_caps()
            )
            .is_ok()
    );
    let before = topology.get(ROOT_CORE);

    assert_eq!(
        topology.mark_hardware_online(ROOT_CORE, ROOT_CORE).err(),
        Some(CoreError::InvalidStateTransition)
    );
    assert_eq!(topology.get(ROOT_CORE), before);
}

#[test]
fn core_topology_quarantine_is_reachable_and_terminal() {
    let mut topology = match CoreTopology::<1>::new(ROOT_CORE) {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };
    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                ROOT_CORE,
                CpuHardwareId::new(0),
                qemu_bootstrap_caps()
            )
            .is_ok()
    );
    assert!(topology.quarantine(ROOT_CORE, ROOT_CORE).is_ok());
    let Some(entry) = topology.get(ROOT_CORE) else {
        return assert_eq!(Some(CoreError::UnknownCore), None);
    };
    assert_eq!(entry.hardware_state(), CoreHardwareState::Quarantined);
    assert_eq!(entry.local_state(), CoreState::Quarantined);

    assert_eq!(
        topology.quarantine(ROOT_CORE, ROOT_CORE).err(),
        Some(CoreError::InvalidStateTransition)
    );
    assert_eq!(
        topology.stage_startup(ROOT_CORE, ROOT_CORE).err(),
        Some(CoreError::InvalidStateTransition)
    );
    assert_eq!(
        topology
            .assign_role(ROOT_CORE, ROOT_CORE, CoreRole::Bootstrap)
            .err(),
        Some(CoreError::InvalidStateTransition)
    );
}

#[test]
fn core_topology_status_excludes_quarantined_assigned_roles() {
    let mut topology = match CoreTopology::<1>::new(ROOT_CORE) {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };
    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                ROOT_CORE,
                CpuHardwareId::new(0),
                qemu_bootstrap_caps()
            )
            .is_ok()
    );
    assert!(
        topology
            .assign_role(ROOT_CORE, ROOT_CORE, CoreRole::Bootstrap)
            .is_ok()
    );

    let assigned_status = topology.status();
    assert_eq!(assigned_status.assigned(), 1);
    assert_eq!(assigned_status.bootstrap_roles(), 1);

    assert!(topology.quarantine(ROOT_CORE, ROOT_CORE).is_ok());

    let status = topology.status();
    assert_eq!(status.discovered(), 1);
    assert_eq!(status.hardware_online(), 0);
    assert_eq!(status.assigned(), 0);
    assert_eq!(status.bootstrap_roles(), 0);
    assert_eq!(status.scheduler_roles(), 0);
    assert_eq!(status.driver_service_roles(), 0);
    assert_eq!(status.idle_roles(), 0);
}

#[test]
fn core_topology_does_not_count_unassigned_idle_cores() {
    let mut topology = match CoreTopology::<1>::new(ROOT_CORE) {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };
    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                ROOT_CORE,
                CpuHardwareId::new(0),
                qemu_bootstrap_caps()
            )
            .is_ok()
    );
    assert_eq!(topology.status().idle_roles(), 0);

    assert!(
        topology
            .assign_role(ROOT_CORE, ROOT_CORE, CoreRole::Idle)
            .is_ok()
    );
    assert_eq!(topology.status().idle_roles(), 1);
}

#[test]
fn core_topology_mutation_requires_owner_core() {
    let mut topology = match CoreTopology::<1>::new(ROOT_CORE) {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert_eq!(
        topology
            .insert_discovered(
                CoreId::new(99),
                ROOT_CORE,
                CpuHardwareId::new(0),
                qemu_bootstrap_caps(),
            )
            .err(),
        Some(CoreError::OwnerMismatch)
    );
    assert_eq!(topology.status().discovered(), 0);
}
