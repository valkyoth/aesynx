use aesynx_abi::{CoreId, CpuHardwareId, ROOT_CORE};

use crate::{
    BootBarrier, CoreAssignmentState, CoreCapabilitySet, CoreError, CoreHardwareState, CoreIsa,
    CoreLocal, CorePerformanceClass, CoreRegistry, CoreRole, CoreState, CoreTopology,
};

mod topology;

fn qemu_bootstrap_caps() -> CoreCapabilitySet {
    CoreCapabilitySet::new(CoreIsa::X86_64, CorePerformanceClass::Control)
        .with_local_timer(true)
        .with_ipi(true)
        .with_directed_irq(true)
        .with_shared_memory_atomics(true)
}

fn qemu_worker_caps(performance_class: CorePerformanceClass) -> CoreCapabilitySet {
    CoreCapabilitySet::new(CoreIsa::X86_64, performance_class)
        .with_local_timer(true)
        .with_ipi(true)
        .with_directed_irq(true)
        .with_shared_memory_atomics(true)
}

#[test]
fn core_roles_expose_amp_authority_classes() {
    assert_eq!(CoreRole::Bootstrap.label(), "bootstrap");
    assert!(CoreRole::Bootstrap.can_schedule_tasks());
    assert!(CoreRole::Bootstrap.can_own_driver_irq());
    assert!(CoreRole::Scheduler.can_schedule_tasks());
    assert!(!CoreRole::Scheduler.can_own_driver_irq());
    assert!(CoreRole::DriverService.can_own_driver_irq());
    assert!(!CoreRole::Idle.can_schedule_tasks());
}

#[test]
fn core_capabilities_record_heterogeneous_metadata() {
    let caps = CoreCapabilitySet::new(CoreIsa::Aarch64, CorePerformanceClass::Efficiency)
        .with_local_timer(true);

    assert_eq!(caps.isa(), CoreIsa::Aarch64);
    assert_eq!(caps.isa().label(), "aarch64");
    assert_eq!(caps.performance_class(), CorePerformanceClass::Efficiency);
    assert_eq!(caps.performance_class().label(), "efficiency");
    assert!(caps.has_local_timer());
    assert!(!caps.supports_ipi());
}

#[test]
fn core_local_tracks_role_state_and_telemetry() {
    let mut local = CoreLocal::new(
        ROOT_CORE,
        CoreRole::Idle,
        qemu_bootstrap_caps(),
        CoreState::Booting,
    );

    assert!(local.is_live());
    assert!(local.assign_role(CoreRole::Bootstrap).is_ok());
    assert!(local.mark_online().is_ok());
    assert!(local.telemetry_mut().record_local_event().is_ok());

    assert_eq!(local.id(), ROOT_CORE);
    assert_eq!(local.role(), CoreRole::Bootstrap);
    assert_eq!(local.state(), CoreState::Online);
    assert_eq!(local.telemetry().role_assignments(), 1);
    assert_eq!(local.telemetry().local_events(), 1);
}

#[test]
fn core_registry_is_owner_scoped_and_rejects_duplicates() {
    let local = CoreLocal::new(
        ROOT_CORE,
        CoreRole::Bootstrap,
        qemu_bootstrap_caps(),
        CoreState::Online,
    );
    let mut registry = match CoreRegistry::<2>::new(ROOT_CORE) {
        Ok(registry) => registry,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert!(registry.insert(ROOT_CORE, local).is_ok());
    assert_eq!(
        registry.insert(ROOT_CORE, local).err(),
        Some(CoreError::DuplicateCore)
    );
    assert_eq!(registry.live_count(), 1);
    assert_eq!(registry.status().owner_core(), ROOT_CORE);
    assert_eq!(registry.status().len(), 1);
    assert_eq!(registry.status().capacity(), 2);
    assert_eq!(registry.status().epoch(), 1);
    assert!(
        registry
            .require_role(ROOT_CORE, CoreRole::Bootstrap)
            .is_ok()
    );
    assert_eq!(
        registry
            .require_role(ROOT_CORE, CoreRole::DriverService)
            .err(),
        Some(CoreError::RoleMismatch)
    );
    assert_eq!(
        registry.require_role(CoreId::new(9), CoreRole::Idle).err(),
        Some(CoreError::UnknownCore)
    );
}

#[test]
fn core_registry_insert_rejects_epoch_overflow_without_mutation() {
    let local = CoreLocal::new(
        ROOT_CORE,
        CoreRole::Bootstrap,
        qemu_bootstrap_caps(),
        CoreState::Online,
    );
    let mut registry = match CoreRegistry::<2>::new(ROOT_CORE)
        .map(|registry| registry.with_epoch_for_test(u64::MAX))
    {
        Ok(registry) => registry,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert_eq!(
        registry.insert(ROOT_CORE, local).err(),
        Some(CoreError::TelemetryOverflow)
    );
    assert_eq!(registry.status().len(), 0);
    assert_eq!(registry.status().epoch(), u64::MAX);
    assert!(!registry.contains(ROOT_CORE));
    assert_eq!(registry.live_count(), 0);
}

#[test]
fn boot_barrier_is_validate_then_commit() {
    let mut barrier = match BootBarrier::<2>::new(ROOT_CORE) {
        Ok(barrier) => barrier,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert_eq!(
        barrier.arrive(ROOT_CORE).err(),
        Some(CoreError::BarrierNotSealed)
    );
    assert!(barrier.add_participant(ROOT_CORE, ROOT_CORE).is_ok());
    assert_eq!(
        barrier.add_participant(ROOT_CORE, ROOT_CORE).err(),
        Some(CoreError::DuplicateCore)
    );
    assert_eq!(barrier.status().participants(), 1);
    assert_eq!(barrier.status().arrivals(), 0);
    assert!(barrier.seal(ROOT_CORE).is_ok());
    assert_eq!(
        barrier.add_participant(ROOT_CORE, CoreId::new(1)).err(),
        Some(CoreError::BarrierSealed)
    );
    assert_eq!(
        barrier.arrive(CoreId::new(1)).err(),
        Some(CoreError::UnknownCore)
    );
    assert!(barrier.arrive(ROOT_CORE).is_ok());
    assert_eq!(
        barrier.arrive(ROOT_CORE).err(),
        Some(CoreError::AlreadyArrived)
    );

    let status = barrier.status();
    assert!(status.sealed());
    assert!(status.all_arrived());
    assert_eq!(status.owner_core(), ROOT_CORE);
}

#[test]
fn registry_and_barrier_setup_require_owner_core() {
    let local = CoreLocal::new(
        ROOT_CORE,
        CoreRole::Bootstrap,
        qemu_bootstrap_caps(),
        CoreState::Online,
    );
    let mut registry = match CoreRegistry::<1>::new(ROOT_CORE) {
        Ok(registry) => registry,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };
    assert_eq!(
        registry.insert(CoreId::new(99), local).err(),
        Some(CoreError::OwnerMismatch)
    );
    assert!(registry.status().is_empty());

    let mut barrier = match BootBarrier::<1>::new(ROOT_CORE) {
        Ok(barrier) => barrier,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };
    assert_eq!(
        barrier.add_participant(CoreId::new(99), ROOT_CORE).err(),
        Some(CoreError::OwnerMismatch)
    );
    assert_eq!(
        barrier.seal(CoreId::new(99)).err(),
        Some(CoreError::OwnerMismatch)
    );
    assert!(barrier.status().is_empty());
}

#[test]
fn core_topology_tracks_qemu_four_core_ownership() {
    let mut topology = match CoreTopology::<4>::new(ROOT_CORE) {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                ROOT_CORE,
                CpuHardwareId::new(0),
                qemu_bootstrap_caps(),
            )
            .is_ok()
    );
    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                CoreId::new(1),
                CpuHardwareId::new(1),
                qemu_worker_caps(CorePerformanceClass::Performance),
            )
            .is_ok()
    );
    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                CoreId::new(2),
                CpuHardwareId::new(2),
                qemu_worker_caps(CorePerformanceClass::Control),
            )
            .is_ok()
    );
    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                CoreId::new(3),
                CpuHardwareId::new(3),
                qemu_worker_caps(CorePerformanceClass::Efficiency),
            )
            .is_ok()
    );

    let root_ticket = match topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE) {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let scheduler_ticket = match topology.stage_startup_ticket(ROOT_CORE, CoreId::new(1)) {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let driver_ticket = match topology.stage_startup_ticket(ROOT_CORE, CoreId::new(2)) {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let idle_ticket = match topology.stage_startup_ticket(ROOT_CORE, CoreId::new(3)) {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };
    assert!(
        topology
            .assign_role(ROOT_CORE, ROOT_CORE, CoreRole::Bootstrap)
            .is_ok()
    );
    assert!(
        topology
            .assign_role(ROOT_CORE, CoreId::new(1), CoreRole::Scheduler)
            .is_ok()
    );
    assert!(
        topology
            .assign_role(ROOT_CORE, CoreId::new(2), CoreRole::DriverService)
            .is_ok()
    );
    assert!(
        topology
            .assign_role(ROOT_CORE, CoreId::new(3), CoreRole::Idle)
            .is_ok()
    );
    let root_arrival = match root_ticket.observe_arrival(ROOT_CORE, CpuHardwareId::new(0)) {
        Ok(arrival) => arrival,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let scheduler_arrival =
        match scheduler_ticket.observe_arrival(CoreId::new(1), CpuHardwareId::new(1)) {
            Ok(arrival) => arrival,
            Err(error) => return assert_eq!(Some(error), None),
        };
    let driver_arrival = match driver_ticket.observe_arrival(CoreId::new(2), CpuHardwareId::new(2))
    {
        Ok(arrival) => arrival,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let idle_arrival = match idle_ticket.observe_arrival(CoreId::new(3), CpuHardwareId::new(3)) {
        Ok(arrival) => arrival,
        Err(error) => return assert_eq!(Some(error), None),
    };

    assert!(
        topology
            .mark_hardware_online(ROOT_CORE, root_arrival)
            .is_ok()
    );
    assert!(
        topology
            .mark_hardware_online(ROOT_CORE, scheduler_arrival)
            .is_ok()
    );
    assert!(
        topology
            .mark_hardware_online(ROOT_CORE, driver_arrival)
            .is_ok()
    );
    assert!(
        topology
            .mark_hardware_online(ROOT_CORE, idle_arrival)
            .is_ok()
    );

    let status = topology.status();
    assert_eq!(status.owner_core(), ROOT_CORE);
    assert_eq!(status.discovered(), 4);
    assert_eq!(status.hardware_online(), 4);
    assert_eq!(status.assigned(), 4);
    assert_eq!(status.bootstrap_roles(), 1);
    assert_eq!(status.scheduler_roles(), 1);
    assert_eq!(status.driver_service_roles(), 1);
    assert_eq!(status.idle_roles(), 1);
    assert_eq!(status.capacity(), 4);

    let Some(driver_core) = topology.get(CoreId::new(2)) else {
        return assert_eq!(Some(CoreError::UnknownCore), None);
    };
    assert_eq!(driver_core.hardware_id(), CpuHardwareId::new(2));
    assert_eq!(driver_core.hardware_state(), CoreHardwareState::Online);
    assert_eq!(
        driver_core.assignment_state(),
        CoreAssignmentState::Assigned
    );
    assert_eq!(driver_core.local_state(), CoreState::Online);
    assert_eq!(driver_core.telemetry().role_assignments(), 1);
    assert_eq!(driver_core.telemetry().local_events(), 2);
}

#[test]
fn core_topology_rejects_duplicate_hardware_ids_without_mutation() {
    let mut topology = match CoreTopology::<2>::new(ROOT_CORE) {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                ROOT_CORE,
                CpuHardwareId::new(7),
                qemu_bootstrap_caps(),
            )
            .is_ok()
    );
    let before = topology.status();

    assert_eq!(
        topology
            .insert_discovered(
                ROOT_CORE,
                CoreId::new(1),
                CpuHardwareId::new(7),
                qemu_worker_caps(CorePerformanceClass::Performance),
            )
            .err(),
        Some(CoreError::DuplicateHardwareId)
    );
    assert_eq!(topology.status(), before);
    assert!(topology.get(CoreId::new(1)).is_none());
}

#[test]
fn core_topology_failed_transition_keeps_state_unchanged() {
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
                qemu_bootstrap_caps(),
            )
            .is_ok()
    );
    let _ticket = match topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE) {
        Ok(ticket) => ticket,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let before = topology.status();
    let entry_before = topology.get(ROOT_CORE);

    assert_eq!(
        topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE).err(),
        Some(CoreError::InvalidStateTransition)
    );
    assert_eq!(topology.status(), before);
    assert_eq!(topology.get(ROOT_CORE), entry_before);
}

#[test]
fn core_topology_epoch_overflow_rejects_commit() {
    let mut topology = match CoreTopology::<1>::new(ROOT_CORE)
        .map(|topology| topology.with_epoch_for_test(u64::MAX))
    {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert_eq!(
        topology
            .insert_discovered(
                ROOT_CORE,
                ROOT_CORE,
                CpuHardwareId::new(0),
                qemu_bootstrap_caps(),
            )
            .err(),
        Some(CoreError::TelemetryOverflow)
    );
    assert_eq!(topology.status().discovered(), 0);
    assert!(topology.get(ROOT_CORE).is_none());
}
