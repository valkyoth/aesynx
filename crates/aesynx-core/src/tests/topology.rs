use aesynx_abi::{CoreId, CpuHardwareId, ROOT_CORE};
use core::fmt::{self, Write};

use crate::{CoreError, CoreHardwareState, CoreRole, CoreStartupArrival, CoreState, CoreTopology};

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
    let ticket = match topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE) {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };
    assert!(
        topology
            .assign_role(ROOT_CORE, ROOT_CORE, CoreRole::Bootstrap)
            .is_ok()
    );
    let arrival = match ticket.observe_arrival(ROOT_CORE, CpuHardwareId::new(0)) {
        Ok(arrival) => arrival,
        Err(error) => return assert_eq!(Some(error), None),
    };
    assert!(topology.mark_hardware_online(ROOT_CORE, arrival).is_ok());
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
fn core_topology_rejects_online_quarantine_without_fault_path() {
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
    let ticket = match topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE) {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let arrival = match ticket.observe_arrival(ROOT_CORE, CpuHardwareId::new(0)) {
        Ok(arrival) => arrival,
        Err(error) => return assert_eq!(Some(error), None),
    };
    assert!(topology.mark_hardware_online(ROOT_CORE, arrival).is_ok());
    let before = topology.get(ROOT_CORE);

    assert_eq!(
        topology.quarantine(ROOT_CORE, ROOT_CORE).err(),
        Some(CoreError::InvalidStateTransition)
    );
    assert_eq!(topology.get(ROOT_CORE), before);
}

#[test]
fn core_topology_arrival_evidence_requires_matching_ticket() {
    let ticket = match staged_single_core_ticket() {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };

    assert_eq!(
        ticket
            .observe_arrival(ROOT_CORE, CpuHardwareId::new(99))
            .err(),
        Some(CoreError::StartupEvidenceMismatch)
    );

    let ticket = match staged_single_core_ticket() {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };
    assert_eq!(
        ticket
            .observe_arrival(CoreId::new(99), CpuHardwareId::new(0))
            .err(),
        Some(CoreError::StartupEvidenceMismatch)
    );
}

#[test]
fn core_topology_rejects_arrival_epoch_that_does_not_match_staging() {
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
    let ticket = match topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE) {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let stale_arrival = CoreStartupArrival::new(ROOT_CORE, CpuHardwareId::new(0), ROOT_CORE, 0);
    let future_arrival =
        CoreStartupArrival::new(ROOT_CORE, CpuHardwareId::new(0), ROOT_CORE, u64::MAX);
    let before = topology.get(ROOT_CORE);

    assert_eq!(
        topology
            .mark_hardware_online(ROOT_CORE, stale_arrival)
            .err(),
        Some(CoreError::InvalidStartupEpoch)
    );
    assert_eq!(topology.get(ROOT_CORE), before);
    assert_eq!(
        topology
            .mark_hardware_online(ROOT_CORE, future_arrival)
            .err(),
        Some(CoreError::InvalidStartupEpoch)
    );
    assert_eq!(topology.get(ROOT_CORE), before);

    let arrival = match ticket.observe_arrival(ROOT_CORE, CpuHardwareId::new(0)) {
        Ok(arrival) => arrival,
        Err(error) => return assert_eq!(Some(error), None),
    };
    assert!(topology.mark_hardware_online(ROOT_CORE, arrival).is_ok());
}

#[test]
fn core_startup_ticket_debug_redacts_hardware_id_and_epoch() {
    let mut topology = match CoreTopology::<1>::new(ROOT_CORE) {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };
    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                ROOT_CORE,
                CpuHardwareId::new(0xfeed_beef),
                qemu_bootstrap_caps()
            )
            .is_ok()
    );
    let ticket = match topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE) {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };

    let mut ticket_debug = FixedDebugBuffer::new();
    assert!(write!(&mut ticket_debug, "{ticket:?}").is_ok());
    let arrival = match ticket.observe_arrival(ROOT_CORE, CpuHardwareId::new(0xfeed_beef)) {
        Ok(arrival) => arrival,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let mut arrival_debug = FixedDebugBuffer::new();
    assert!(write!(&mut arrival_debug, "{arrival:?}").is_ok());

    assert!(
        ticket_debug
            .as_str()
            .contains("hardware_id: \"<redacted>\"")
    );
    assert!(
        ticket_debug
            .as_str()
            .contains("startup_epoch: \"<redacted>\"")
    );
    assert!(!ticket_debug.as_str().contains("feed"));
    assert!(!ticket_debug.as_str().contains("48879"));
    assert!(
        arrival_debug
            .as_str()
            .contains("hardware_id: \"<redacted>\"")
    );
    assert!(
        arrival_debug
            .as_str()
            .contains("startup_epoch: \"<redacted>\"")
    );
    assert!(!arrival_debug.as_str().contains("feed"));
    assert!(!arrival_debug.as_str().contains("48879"));
}

fn staged_single_core_ticket() -> Result<crate::CoreStartupTicket, CoreError> {
    let mut topology = CoreTopology::<1>::new(ROOT_CORE)?;
    topology.insert_discovered(
        ROOT_CORE,
        ROOT_CORE,
        CpuHardwareId::new(0),
        qemu_bootstrap_caps(),
    )?;
    topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE)
}

struct FixedDebugBuffer {
    bytes: [u8; 256],
    len: usize,
}

impl FixedDebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 256],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or_default()
    }
}

impl Write for FixedDebugBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
        if end > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
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
        topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE).err(),
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
